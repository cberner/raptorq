import os
import random
import subprocess
import sys
import sysconfig
import threading
from concurrent.futures import ThreadPoolExecutor
from unittest import TestCase, skipIf


def _check_gil_release(operation):
    from raptorq import Encoder, Decoder

    data = b"raptorq!" * (128 * 1024) + b"!"
    symbol_size = 8192
    if operation == "setup":
        def work():
            return Encoder.with_defaults(data, symbol_size)
    else:
        encoder = Encoder.with_defaults(data, symbol_size)
        if operation == "packets":
            def work():
                return encoder.get_encoded_packets(256)
        else:
            packets = encoder.get_encoded_packets(1)
            source_count = (len(data) + symbol_size - 1) // symbol_size
            decoder = Decoder.with_defaults(len(data), symbol_size)
            # Leave one source symbol missing so the measured call must solve
            # the coding system, rather than just concatenate source symbols.
            for packet in packets[1:source_count]:
                assert decoder.decode(packet) is None

            def work():
                return decoder.decode(packets[source_count])

    ready = threading.Event()
    stop = threading.Event()
    gate = threading.Lock()
    heartbeats = []
    borrow_errors = []

    def heartbeat():
        ready.set()
        with gate:
            if operation == "decode":
                try:
                    decoder.decode(packets[0])
                except RuntimeError as error:
                    borrow_errors.append(error)
            while not stop.is_set():
                heartbeats.append(None)
                stop.wait(0.001)

    # Wake a waiting thread immediately before the codec call. Suppress normal
    # Python switching so it can run only if native code releases the GIL.
    gate.acquire()
    thread = threading.Thread(target=heartbeat, daemon=True)
    thread.start()
    assert ready.wait(5)
    previous_interval = sys.getswitchinterval()
    try:
        sys.setswitchinterval(1000)
        gate.release()
        result = work()
        progressed = bool(heartbeats)
    finally:
        sys.setswitchinterval(previous_interval)
        stop.set()
        thread.join(5)

    assert not thread.is_alive(), "heartbeat thread did not stop"
    assert progressed, operation + " held the GIL"
    if operation == "decode":
        assert result == data
        assert len(borrow_errors) == 1, "overlapping decoder call was not rejected"


class EncoderDecoderTestCase(TestCase):
    def test_source_roundtrip_with_partial_symbol(self):
        from raptorq import Encoder, Decoder

        data = bytes(range(256)) * 8 + b"partial"
        packets = Encoder.with_defaults(data, 512).get_encoded_packets(0)
        self.assertIsInstance(packets, list)
        self.assertEqual(len(packets), 5)
        self.assertTrue(all(isinstance(packet, bytes) for packet in packets))
        decoder = Decoder.with_defaults(len(data), 512)
        for packet in packets[:-1]:
            self.assertIsNone(decoder.decode(packet))
        self.assertEqual(decoder.decode(packets[-1]), data)

    def test_repair_roundtrip_with_partial_symbol(self):
        from raptorq import Encoder, Decoder

        data = bytes(range(256)) * 8 + b"partial"
        packets = Encoder.with_defaults(data, 512).get_encoded_packets(16)
        decoder = Decoder.with_defaults(len(data), 512)
        # Discard every source packet and recover using only repair packets.
        result = None
        for packet in packets[5:]:
            result = decoder.decode(packet)
            if result is not None:
                break
        self.assertEqual(result, data)

    def test_independent_codecs_in_executor(self):
        from raptorq import Encoder, Decoder

        def roundtrip(index):
            data = bytes([index]) * 4097
            encoder = Encoder.with_defaults(data, 512)
            decoder = Decoder.with_defaults(len(data), 512)
            result = None
            for packet in encoder.get_encoded_packets(4):
                result = decoder.decode(packet)
                if result is not None:
                    break
            return result, data

        with ThreadPoolExecutor(max_workers=4) as executor:
            for result, data in executor.map(roundtrip, range(8)):
                self.assertEqual(result, data)

    @skipIf(sysconfig.get_config_var("Py_GIL_DISABLED"),
            "requires a GIL-enabled CPython build")
    def test_codec_releases_gil(self):
        for operation in ("setup", "packets", "decode"):
            with self.subTest(operation=operation):
                process = subprocess.run(
                    [sys.executable, "-c",
                     "from test import _check_gil_release; "
                     "_check_gil_release(" + repr(operation) + ")"],
                    cwd=os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
                    capture_output=True,
                    text=True,
                    timeout=120,
                )
                self.assertEqual(process.returncode, 0,
                                 process.stdout + process.stderr)

    def test_encoder(self):
        from raptorq import Encoder

        data = os.urandom(1024)
        encoder = Encoder.with_defaults(data, 512)
        packets = encoder.get_encoded_packets(42)

        self.assertIsInstance(packets, list)
        self.assertGreater(len(packets), 0)
        for packet in packets:
            self.assertIsInstance(packet, bytes)

    def test_decoder(self):
        from raptorq import Encoder, Decoder

        data = os.urandom(1024)
        encoder = Encoder.with_defaults(data, 512)
        packets = encoder.get_encoded_packets(42)

        random.shuffle(packets)

        decoded_data = None
        decoder = Decoder.with_defaults(len(data), 512)
        for packet in packets:
            decoded_data = decoder.decode(packet)
            if decoded_data is not None:
                break

        self.assertEqual(decoded_data, data)
