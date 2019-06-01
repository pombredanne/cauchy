import socket
from transaction import Transaction

class NativeClient:
    def __init__(self, ip: str, port: int):
        self.ip = ip
        self.port = port
        self.socket = socket.socket()

    def connect(self):
        self.socket.connect((self.ip, self.port))

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
        value = self.socket.recv(value_size)

        return value
