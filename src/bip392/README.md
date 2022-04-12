<pre>
  BIP: 392
  Layer: Applications
  Title: Methodology for script wallet backups
  Author: Vishal Menon <vishalmenon.92@gmail.com>
  Comments-Summary: No Comments yet
  Comments-URI: https://github.com/bitcoin/bips/wiki/Comments:BIP-0392
  Status: Draft
  Type: Process Track
  Created: 2022-04-12
</pre>

# Copyright

This proposal is hereby placed in the public domain.

## Table of Contents
- [Abstract](#abstract)
- [Motivation](#motivation)
- [Encryption Standard](#estd)
- [Key Source Construction](#ksc)
- [Derivation Scheme](#ds)
- [External Recovery Data](#erd)
- [Data Redundancy](#datared)
- [Implementations](#impl)
- [Final Interface](#finint)

## Abstract

This BIP describes an encryption standard and a method for handling script wallet backups using the familiar interface of BIP39 mnemonic seed and BIP32 key management.

## Motivation

Most users of Bitcoin are comfortable with using a single mnemonic as an interface for key management and wallet recovery. For single signature wallets, this works for most cases, except where users use non-standard BIP32 derivation paths. Even considering non-standard paths, most single signature wallets can be recovered via brute force with the mnemonic alone. 

Script wallets that require more than just a single key to satisfy i.e. contain information outside the mnemonic words that are required to recover the wallet, present an obstcle in the adoption of script wallets due to difficulty in backup and recovery.

Several attempts have been made at the problem and no solution preserves the single mnemonic interface.

Most solutions have realized that an encryption standard is required. 

The solutions differ in the key source used in encryption and most have either abandoned the mnemonic interface or added another layer that requires writing down another key in a different format to avoid confusion. Either way, still more data to keep secret.

Our goal is to construct a methodology for script wallet backup that maintains the single mnemonic interface to recover single signature AND script wallets with minimized trade-offs.

## Encryption Standard

We propose primarily supporting the simple ChaCha20Poly1305 cipher standard, with the requirement of a 24-bit initialization vector or nonce.

Ciphertext is encoded in base64 to be as compact as possible.

Example : ```"ZXh0cmEgbG9uZyB1bmlxdWUgbm9uY2Uh":"+dj7i1ViaGhL/3PvsAewrqoIqZBv3D3ACDxPbQWaioGIrqYyqRHuyVybXlUmdHiVK3To38omnU/3ujE6xFfHCAo="```

Other algorithms such as AES256-CBC/GCM etc. can also be supported with very little effort.

## Key Source Construction

The key used for encryption will be a `sha256(privatekey)` derived from the mnemonic seed based on a derivation scheme.

## Derivation Scheme

We propose using the following derivation scheme:

1. Purpose(encryption): 392'
2. Network(bitcoin): 0' 
3. Account: *'

Giving a standard derivation path of `m/392'/0'/*'` 

All paths are hardened.

If this mnemonic is part of several different scripts, each will have their corresponding * account' key.

Wallets are free to use any path of their choice OR add more paths to proposed scheme. Wallets should allow users to input their own specified path, in the event that they have chosen to use another scheme.

## External Recovery Data 

Users must be made explicitly aware of the fact that script wallets require external data (erd) that is not contained within their mnemonic.

The external recovery data (erd) being encrypted is the `public script descriptor`; which contains all the information required for an individual to recover their script wallet. The corresponding private data can be extracted from the mnemonic.

We encourage not using private descriptors as data; for better layered security. However, this is a tradeoff that wallets can decide to make for convenience or user experience.

## Data Redundancy
 
The mnemonic only supports the erd in being redundant and highly available; through encryption.

Since the erd is encrypted, more copies makes recovery safer and easier.

Users and wallets must then focus on making and sharing as many copies of their erd as part of the wallet backup process.

## Final Interface

The final interface for the user:

- Generate a mnemonic (on some hardware)
- Write it down (on a seedplate? or in your head?)
- Tag it with `bip392` (on the same seedplate or in your head.)

This mnemonic is now ready to take part in scripts.

To avoid confusion, it is encouraged not to tag a seed as bip392 compatible if only being used for single signature wallets.

Whenever you use a public key from this mnemonic in a script, you create a dedicated account number for it and encrypt the `public script descriptor as erd` and create multiple copies of it.

Users of air-gapped hardware only require their manufacturers to support the `encrypt/decrypt functions` in order to facilitate much more reliable bitcoin scripting on the application layer.

## Implementations

`stackmate-core` - contains examples for 3 different script types.

