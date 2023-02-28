from shadow_drive import ShadowDriveClient
from solders.keypair import Keypair
import argparse

parser = argparse.ArgumentParser()
parser.add_argument('--keypair', metavar='keypair', type=str, required=True, 
                    help='The keypair file to use (e.g. keypair.json, dev.json)')
args = parser.parse_args()

# Initialize client
client = ShadowDriveClient(args.keypair)
print("Initialized client")

# Create account
size = 2 ** 20
account, tx = client.create_account("full_test", size, use_account=True)
print(f"Created storage account {account}")

# Upload files
files = ["./files/alpha.txt", "./files/not_alpha.txt"]
urls = client.upload_files(files)
print("Uploaded files")

# Add and Reduce Storage
client.add_storage(2**20)
client.reduce_storage(2**20)

# Get file
current_files = client.list_files()
file = client.get_file(current_files[0])
print(f"got file {file}")

# Delete files
client.delete_files(urls)
print("Deleted files")

# Delete account
client.delete_account(account)
print("Closed account")
