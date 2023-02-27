# The Python SDK for Shadow Drive
###### By: GenesysGo

### Getting Started
Check out the `examples/` directory for a demonstration of the functionality.
```python
from shadow_drive import ShadowDriveClient

# Initialize client
client = ShadowDriveClient("test.json")

# Create account
size = 2 ** 20
account, tx = client.create_account("test", size, use_account=True)

# Upload files
files = ["./files/alpha.txt", "./files/not_alpha.txt"]
urls = client.upload_files(files)

# Delete files
client.delete_files(urls)

# Delete account
client.delete_account(account)
```

### About this Repo
This package uses PyO3 to build a wrapper around the official Shadow Drive Rust SDK. For more information, see the Rust SDK documentation.