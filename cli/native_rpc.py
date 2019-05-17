import socket
from time import time


class Transaction:
    def __init__(timestamp: int = None, aux: bytes = None, aux_path: str = None, binary: bytes = None, binary_path: str = None):
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

    def encode(raw: bytes):
        time_vi = encode_varint(int(time() * 1_000))
        aux_len = encode_varint(len(aux))
        bin_len = encode_varint(len(binary))
        msg = time_vi + aux_len + aux + bin_len + binary
        return msg

    @staticmethod
    def decode(raw: bytes):
        return  # TODO


class NativeClient:
    def __init__(self, address: str):
        self.address = address
        self.socket = socket.socket()

    def connect(self):
        self.socket.connect(self.address)

    def close(self):
        self.socket.close()

    def add_peer(self, ip: str, port: int):
        ip_bytes = bytes(map(int, ip.split(".")))
        msg = b"\x00" + ip_bytes + int(port).to_bytes(2, byteorder="big")
        self.socket.send(msg)

        if self.socket.recv(1) == b"\x01":
            raise Exception("failed to add peer")
        elif self.socket.recv(1) != b"\x00":
            raise Exception("unexpected response")

    def add_transaction(self, tx: Transaction):
        msg = b"\x01" + tx.encode()
        self.socket.send(msg)

        if self.socket.recv(1) == b"\x01":
            raise Exception("failed to add transaction")
        elif self.socket.recv(1) != b"\x00":
            raise Exception("unexpected response")

    def fetch_value(self, key: bytes):
        msg = b"\x02" + key
        self.socket.send(msg)

        ret_val = self.socket.recv(1)

        if ret_val == b"\x01":
            raise Exception("failed to fetch value")
        elif ret_val != b"\x02":
            raise Exception("unexpected response")

        value_size = int.from_bytes(self.socket.recv(8), "big")
        value = self.socket.recv(vaue_size)

        return value
