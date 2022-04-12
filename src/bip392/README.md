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
- [Key Derivation](#kd)
- [Encryption Algorithm](#encryptalgo)
- [Public Script Descriptor](#pubdes)
- [Data Redundancy](#datared)
- [Notes](#notes)
- [Implementations](#implementations)

## Abstract

This BIP describes an encryption standard and a method of generating keys from a BIP39 mnemonic seed and BIP32 for key management.

## Motivation

Most users of Bitcoin are familiar with the mnemonic interface for key management and wallet recovery. For single signature wallets, this works for most cases, except where users use non-standard BIP32 derivation paths. Even considering non-standard paths, most single signature wallets can be recovered via brute force with the mnemonic alone. 

Script wallets that require more than just a single key to satisfy i.e. contain information outside the mnemonic words that are required to recover the wallet, present an obstcle in the adoption of script wallets due to difficulty in backup and recovery.

The primary use-case for encryption is to be able to encrypt all script data that is external to the mnemonic. Being encrypted, this data can be distributed across the internet for redundancy. 

This maintains the familiar key management UI of holding just the mnemonic words and being able to recover ANY wallet.

## Key Derivation

We propose using the following derivation scheme:

1. Purpose(encryption): 392'
2. Network(bitcoin): 0' 
3. Account: *'

Giving a standard derivation path of `m/392'/0'/*'` 

Wallets are free to use any path of their choice OR add more paths to proposed scheme. Wallets should allow users to input their own specified path, in the event that they have chosen to use another scheme.

If this mnemonic is part of several different scripts, each will have their corresponding account key.

The key used will be a sha256 hash of the private key derived at the chosen path.

## Encryption Algorithm

We propose primarily supporting the AES-256-CBC standard, with the requirement of a 16-bit initialization vector.

This is because it is the most commonly used standard for encryption.

Other algorithms such as ChaChaPoly126 etc. can also be supported with very little effort.

## Public Script Descriptor

The data being encrypted is the public script descriptor; which contains all the information required for an individual to recover their script wallet. The corresponding private data can be extracted from the mnemonic.

We encourage not using private descriptors as data; for better layered security. However, this is a tradeoff that wallets can decide to make for convenience or user experience.

## Data Redundancy

Users must still be made aware of the fact that script wallets require external data that is not contained within their mnemonic.

The mnemonic only supports the external data in being redundant and highly available; through encryption.

Wallets must focus on creating redundant copies of encrypted wallet backups.

## Implementations

`stackmate-core` - contains examples for 3 different script types.

