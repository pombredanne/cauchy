import six


def encode_varint(num: int):
    parts = list()
    length = 0
    while True:
        if not length:
            e = 0x00
        else:
            e = 0x80
        parts.append(six.int2byte((num & 0x7f) | e))
        if num <= 0x7f:
            break
        num = (num >> 7) - 1
        length += 1
    return b''.join(reversed(parts))