from time import time
from tools import encode_varint

class Transaction:
    def __init__(self, timestamp: int = None, aux: bytes = None, aux_path: str = None, binary: bytes = None, binary_path: str = None):
        if timestamp is None:
            self.timestamp = int(time() * 1_000)
        else:
            self.timestamp = timestamp

        if aux is None:
            if aux_path is None:
                raise Exception("specify aux data")
            with open(aux_path, 'rb') as f:
                self.aux = f.read()
        else:
            self.aux = aux

        if binary is None:
            if binary_path is None:
                raise Exception("specify binary data")
            with open(binary_path, 'rb') as f:
                self.binary = f.read()
        else:
            self.binary = binary

    def encode(self):
        time_vi = encode_varint(int(time() * 1_000))
        aux_len = encode_varint(len(self.aux))
        bin_len = encode_varint(len(self.binary))
        msg = time_vi + aux_len + self.aux + bin_len + self.binary
        return msg

    @staticmethod
    def decode(raw: bytes):
        return  # TODO