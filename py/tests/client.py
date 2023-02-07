import unittest
from shadow_drive import ShadowDriveClient, print_pubkey, sign_message
from solders.keypair import Keypair

class TestClient(unittest.TestCase):

    def test_init(self):
        keypair = Keypair()
        client = ShadowDriveClient(keypair)
        self.assertEqual(type(client), ShadowDriveClient)

    # Ensures the pubkey returned from the python method is the same as
    # the pubkey returned from the inner keypair handling.
    def test_inner_keypair_handling(self):
        keypair = Keypair()
        self.assertEqual(str(keypair.pubkey()), print_pubkey(keypair))

        sig = str(keypair.sign_message(b"1"*32))
        sig2 = sign_message(keypair, list(b"1"*32))
        self.assertEqual(sig, sig2)



if __name__ == '__main__':
    unittest.main()