import six
from ecdsa import SigningKey, NIST256p
from hashlib import sha256

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


def gen_contract_data():
    sk = SigningKey.generate(curve=NIST256p)

    msg_type = [0x1, 0x0]
    acct_to = [ord('A')] * 32
    amt = [0xA, 0x00, 0, 0, 0, 0, 0, 0]
    pubkey_from = sk.get_verifying_key().to_string()
    data = bytes(msg_type) + bytes(acct_to) + pubkey_from + bytes(amt)
    sig = sk.sign(data, hashfunc=sha256)

    # print(len(msg_type), len(acct_to), len(pubkey_from), len(amt), len(sig))

    payload = data + sig

    pstr = ""

    for b in payload:
        pstr += "{0:02X}".format(b)
    print()

    # Now generate the auxdata for the contract initialization
    hasher = sha256()
    hasher.update(pubkey_from)
    # print(hasher.hexdigest() + "E803000000000000")
    return (hasher.hexdigest() + "E803000000000000", pstr)

print(gen_contract_data())