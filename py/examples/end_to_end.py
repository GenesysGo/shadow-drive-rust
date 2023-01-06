from shadow_drive import ShadowDriveClient
from solders.keypair import Keypair
import argparse

parser = argparse.ArgumentParser()
parser.add_argument('--keypair', metavar='keypair', type=str, required=True, 
                    help='The keypair file to use (e.g. keypair.json, dev.json)')
args = parser.parse_args()

# Get keypair
test_keypair_file = open(args.keypair, "r")
test_keypair_bytes = test_keypair_file.read()
keypair = Keypair(test_keypair_bytes)
print("Loaded keypair")

# Initialize client
client = ShadowDriveClient(keypair)
print("Initialized client")

# Create account
size = 2 ** 20
success = False
while not success:
    try:
        account, tx = client.create_account("test", size)
        print("Created storage account")
        success = True
    except ValueError:
        print("failed to create storage account")

# Upload files
files = ["./files/alpha.txt", "./files/not_alpha.txt"]
urls = client.upload_files(files)
print("Uploaded files")

# Delete files
client.delete_files(urls)
print("Deleted files")

# Close account
client.close_account(account)
print("Closed account")
