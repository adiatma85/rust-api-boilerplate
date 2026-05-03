---
name: sops-secrets
description: Guidelines for managing encrypted application secrets and configurations using SOPS and Makefile commands. Use when updating database credentials, API keys, or environment settings.
---

# SOPS Secrets Management

This skill provides instructions for safely managing encrypted configuration files in this project.

## Core Commands

Configurations are stored in `etc/cfg/` and encrypted using SOPS.

### Local Development
- **Decrypt**: `make decrypt-local` (Decrypts `etc/cfg/conf.local.enc.json` to `etc/cfg/conf.json`).
- **Encrypt**: `make encrypt-local` (Encrypts `etc/cfg/conf.json` back to `etc/cfg/conf.local.enc.json`).
- **Edit Directly**: `make conf-local` (Opens the local encrypted file for editing in your default editor).

### Staging/Production
- Replace `local` with `staging` or `production` in the commands above (e.g., `make decrypt-staging`).

## Security Mandates

1. **NEVER COMMIT RAW SECRETS**: Do not stage or commit `etc/cfg/conf.json`. It is ignored by git for a reason.
2. **NEVER LOG SECRETS**: Avoid printing decrypted configuration values to the console or logs.
3. **ONLY COMMIT ENCRYPTED FILES**: Only commit the `.enc.json` files.

## Workflow for Updating Secrets

1. Run `make decrypt-local` to get the raw `conf.json`.
2. Modify `etc/cfg/conf.json` with the new values.
3. Run `make encrypt-local` to update the encrypted version.
4. Delete `etc/cfg/conf.json` if it's no longer needed (though it's gitignored).
5. Stage and commit the changed `etc/cfg/conf.local.enc.json`.
