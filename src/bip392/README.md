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

## Table of Contents
- [Abstract](#abstract)
- [Motivation](#motivation)
- [Encryption Algorithm](#encryptalgo)
- [External Recovery Data](#erd)
- [Data Redundancy](#datared)
- [Key Derivation](#kd)
- [Implementations](#implementations)

## Abstract

This BIP describes an encryption standard and a method for handling script wallet backups using the familiar interface of BIP39 mnemonic seed, BIP32 key management and the concept of data redundancy.

## Motivation

Most users of Bitcoin are comfortable with using a single mnemonic as an interface for key management and wallet recovery. For single signature wallets, this works for most cases, except where users use non-standard BIP32 derivation paths. Even considering non-standard paths, most single signature wallets can be recovered via brute force with the mnemonic alone. 

Script wallets that require more than just a single key to satisfy i.e. contain information outside the mnemonic words that are required to recover the wallet, present an obstcle in the adoption of script wallets due to difficulty in backup and recovery.

Several attempts have been made at the problem and no solution preserves the single mnemonic interface.

Most solutions have realized that an encryption standard is required. 

The solutions differ in the key source used in encryption and most have either abandoned the mnemonic interface or added another layer that requires writing down another key in a different format to avoid confusion. Either way, still more data to keep secret.

Our goal is to construct a methodology for script wallet backup that maintains the single mnemonic interface to recover single signature AND script wallets with minimized trade-offs.

## Encryption Algorithm

We propose primarily supporting the AES-256-CBC standard, with the requirement of a 16-bit initialization vector.

This is because it is the most commonly used standard for encryption.

Other algorithms such as ChaChaPoly126 etc. can also be supported with very little effort.

## External Recovery Data 

The external recovery data (erd) being encrypted is the `public script descriptor`; which contains all the information required for an individual to recover their script wallet. The corresponding private data can be extracted from the mnemonic.

We encourage not using private descriptors as data; for better layered security. However, this is a tradeoff that wallets can decide to make for convenience or user experience.

## Data Redundancy

Users must be made explicitly aware of the fact that script wallets require external data (erd) that is not contained within their mnemonic.

The mnemonic only supports the erd in being redundant and highly available; through encryption.

Users and wallets must then focus on making and sharing as many copies of their erd as part of the wallet backup process.

`the-erd.musbe-red.`

## Key Derivation

The key used will be a sha256 hash of the private key derived from the mnemonic seed based on a derivation scheme.

We propose using the following derivation scheme:

1. Purpose(encryption): 392'
2. Network(bitcoin): 0' 
3. Account: *'

Giving a standard derivation path of `m/392'/0'/*'` 

All paths are hardened.

If this mnemonic is part of several different scripts, each will have their corresponding * account' key.

Wallets are free to use any path of their choice OR add more paths to proposed scheme. Wallets should allow users to input their own specified path, in the event that they have chosen to use another scheme.

## Implementations

`stackmate-core` - contains examples for 3 different script types.

