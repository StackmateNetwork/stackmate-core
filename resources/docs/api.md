## libstackmate api reference

### overview

The libstackmate api can be divided into 3 modules:

1. key:
  - Create mnemonic seed phrase
  - Derive child keys from the root key
  - Schnorr sign/verify messages
  - ECDH shared secrets
  - ChaCha20Poly1305 encrypt/decrypt messages 

2. fee: 
  - Get network fee rate
  - Get weight of a psbt
  - Get absolute fee for a psbt given weight and fee rate

3. wallet:
Can be further divided into 3 parts:
  - recieve: 
      - Generate new addresses (offline)
  - history: 
      - Get transaction history & balance (online)
  - send: 
      - Build, sign and broadcast psbts (online)


### Note on persistent data

libstackmate does not persist any wallet data. Applications that use libstackmate, will be required to implement their own storage for things like:

- descriptors (encrypted)
- used address indexes
- transaction history/balance (for offline reference)

Regarding persistent of private key data, apps MUST:

- encrypt all private data
- only store descriptor strings
- never store a seed/root key; only store extended hardened account keys; derived upto `m/purpose'/network'/account'`


### Note on IO 

#### inputs

In some cases, if inputs are enum types with a only a few possible options (for example: generate_master), if the client uses an invalid string (like length=35), it will default to safe values (like length=24) and continue rather than error.

#### outputs

Outputs are stringified JSON strings. 

The first check should be for an `error` field, if found the stringified JSON will be of the format
```
{
  error: {
    kind: String,
    message: String,
  }
}
```

All other success response is also stringified JSON of types you will see per API in the following example.

## Example API Usage Flow

### Offline

First create a mnemonic master key by specifying `network`, `length` and optional `passphrase`.

```
generate_master(
  network: "test" || "main", (All other strings default to "test")
  length: "12" || "24", (All other strings default to "24")
  passphrase: *const c_char, (Can be empty string)
)->MasterKey {
  fingerprint: String,
  mnemonic: String,
  xprv: String
}
```

Or import a mnemonic to recover your wallet by specifying `network`, `mnemonic` and `passphrase`.

```
import_master(
  network: "test" || "main", (All other strings default to "test")
  mnemonic: *const c_char, (words separated by space)
  passphrase: *const c_char, (Can be empty string)
)->MasterKey {
  fingerprint: String,
  mnemonic: String,
  xprv: String
}
```

Both these functions will output a stringified JSON MasterKey containing a 

- `fingerprint`: an identifier for this seed. *public data*.
- `mnemonic` : the seed phrase that will be displayed to the user. *private. burn after reading.*
- `xprv` : the root xprv for this seed from which account master keys can be derived. *private. burn after deriving.*

*After the user writes down and verifies their mnemonic; erase it from memory before the next step of key derivation.*

*After the key derivation process, erase the root xprv from memory*


#### Key Derivation

The BIP32 standard for key derivation is: `m/purpose'/network'/account'/deposit/index`

This is a standard for key management with bitcoin and closely resembles the unix file path system.

At every path, there exists a key pair. 

`m` represents the source, or the `root` of the path. In our case this is the `fingerprint` - telling us which mnemonic is needed to get the root.

A few rules about path based derivation:

- An `xprv` can keep deriving down a path for all child `xprv` and `xpub` BUT it cannot derive either `xprv` or `xpub` when traversing backwards. 

eg. 

TRUE `m/24'/2'` ->`m/24'/2'/2'`

FALSE `m/24'/2'/2'` -> `m/24'/2'`

*This is why a master key can sign for all children.*

- An `xpub` can keep deriving down a path for a child `xpub` BUT it cannot derive either when traversing backwards.

*This is how watch-only wallets work.*

The `'` represents a hardened path. It can also be represented as `h`. Hardened paths ensure that xpubs CANNOT derive child xpubs down the path. We always harden the first 3 paths in bitcoin and keep the last 2 normal. This gives privacy between accounts and convenience within accounts.

##### Purpose

The first path indicates what type of Script this wallet uses. 

The standard paths are :

- `44h` : Legacy PK scripts which yield `1` addresses
- `49h` : Compatible SH scripts which yield `3` addresses
- `84h` : Native WSH scripts which yield `bc1` addresses

If you want your wallet to support all three types, then technically, each wallet will have to contain 3 descriptors, which use 3 different master keys hardedened at the `purpose` path, derived from the same root.

##### Network

Network is `0h for Mainnet` and `1h for Testnet`.

##### Account

Account is where users can manage different wallets for different use cases.

We start from `0h` and keep incrementing if we need to create more accounts.

##### Deposit(Change)
* Clients and users do not need to worry about correctly setting change indexes. This is handled by the wallet.*

The first unhardened path is to specify the usecase of the next `index` path. 

We use `0 for deposit` or `1 for change`. When we share address with people we use `0: deposit`, when we need an address to get back change, the wallet internall uses `1: change` in this path.

##### Index

The last unhardened path is used to rotate keys for every new address.

Clients must keep track of the last index they use to avoid address reuse. More on this in the `get_address` api.

##### Derive hardened account key

*libstackmate supports two different methods of deriving, suitable for different usecases.* 

For wallets, we recommend using `derive_wallet_account` since it defaults to standard for wallets.

It only requires specifying the `purpose` and `account` number. Infact, even purpose currently locks into 84, even if other values are used. `purpose` only exists to eventually support taproot as an alternative.

```
derive_wallet_account(
    master_xprv: *const c_char,
    purpose: "84", (All other strings default to "84")
    account: *const c_char, (Can be empty - will default to "0" if value cannot be parsed to integer)
)->ChildKeys {
  fingerprint: String,
  hardened_path: String,
  xprv: String,
  xpub: String
}
```


The `ChildKeys` result will now be the `hardened account master key` for a segwit native account.

More `ChildKeys` will be derived from this `xprv` or `xpub` by the wallet to use internally.

The `master_xprv` can now also be discarded from memory.

The following key utils are for non-wallet applications. Can be ignored for now.

BIP32

```
derive_to_path(
    master_xprv: *const c_char,
    derivation_path: *const c_char*,
)->ChildKeys {
  fingerprint: String,
  hardened_path: String,
  xprv: String,
  xpub: String
}
```

```
xprv_to_ec(
  xprv: *const c_char
) -> *mut c_char
```

ECDH 

```
shared_secret(
    local_secret: *const c_char,
    remote_pubkey: *const c_char,
) -> *mut c_char {
```

Schnorr 

```
sign_message(
  message: *const c_char,
  seckey: *const c_char,
) -> *mut c_char
```

```
verify_signature(
  signature: *const c_char,
  message: *const c_char,
  pubkey: *const c_char,
) -> *mut c_char 

```

ChaCha20Poly1305

```
encrypt(
  plaintext: *const c_char,
  seckey: *const c_char,
) -> *mut c_char
```

```
decrypt(
  ciphertext: *const c_char,
  seckey: *const c_char,
) -> *mut c_char 
```

### Policies and descriptors

Now that we have a hardened account key to start a bitcoin wallet, we now move to the `wallet` module set of functions.

Most wallet functions take a `descriptor` as a common input. 

The client must first attempt to create a valid bitcoin policy using their keys and then `compile` it into a wallet `descriptor` string.

The descriptor is a string, with the format:

```
script(conditions)
```

We use only one of the following 2 script types:
- `wpkh` for segwit single sig
- `wsh` for multi-condition segwit scripts

Conditions are usually `keys: pk()`, `timelocks: after()/older()`, `hashlocks: hash160()`.

Where conditions involve `keys`, the extended key format is

```
[source]xkey
```

Key `source` tells us the parent `fingerprint` and the `hardened derivation path` used to reach this child key.

```
[fingerprint/hardened_path]xprv8fWev2sCuSkVWYoNUUSEuqLkmmfiZaVtgxosS5jRE9fw5ejL2odsajv1QyiLrPri3ppgyta6dsFaoDVCF4ZdEAR6qqY4tnaosujsPzLxB49
i.e.
[fingerprint/purpose'/network'/account']xprv8fWev2sCuSkVWYoNUUSEuqLkmmfiZaVtgxosS5jRE9fw5ejL2odsajv1QyiLrPri3ppgyta6dsFaoDVCF4ZdEAR6qqY4tnaosujsPzLxB49
finally, i.e.

[db7d25b5/84'/1'/6']xprv8fWev2sCuSkVWYoNUUSEuqLkmmfiZaVtgxosS5jRE9fw5ejL2odsajv1QyiLrPri3ppgyta6dsFaoDVCF4ZdEAR6qqY4tnaosujsPzLxB49
```

A single sig policy for this account would just wrap the extended key in `pk()`

```
pk([db7d25b5/84'/1'/6']xprv8fWev2sCuSkVWYoNUUSEuqLkmmfiZaVtgxosS5jRE9fw5ejL2odsajv1QyiLrPri3ppgyta6dsFaoDVCF4ZdEAR6qqY4tnaosujsPzLxB49)
```

We add a `/*` to the end of the extended key just to indicate that this `policy` will require more keys derived from the given path onwards. Just like linux paths.

```
pk([db7d25b5/84'/1'/6']xprv8fWev2sCuSkVWYoNUUSEuqLkmmfiZaVtgxosS5jRE9fw5ejL2odsajv1QyiLrPri3ppgyta6dsFaoDVCF4ZdEAR6qqY4tnaosujsPzLxB49/*)
```

If we do not add this, then a descriptor wallet will assume only create a single key wallet, and only generate a single address everytime.

By adding this, the wallet will know to internally derive keys for itself no additional derivation is required by the client.

The above policy will compile into the following descriptor:

```
wpkh([db7d25b5/84'/1'/6']xprv8fWev2sCuSkVWYoNUUSEuqLkmmfiZaVtgxosS5jRE9fw5ejL2odsajv1QyiLrPri3ppgyta6dsFaoDVCF4ZdEAR6qqY4tnaosujsPzLxB49/*)
```

```
compile(
  policy: *const c_char, 
  script_type: "wpkh" || "wsh", (Defaults to "wpkh" for all others)
)->WalletPolicy {
  policy: String,
  descriptor: String
}

```

Now all key data can be removed from memory and only the above descriptor needs to be stored as a `spender` wallet.

If we replace the `xprv` with the corresponding `xpub`, this would become a `watcher` wallet.

Going back from a `watcher` to a `spender` requires `import_master` again, to start from the root key and derive the required account `xprv`.

NOW, we can start using all the main `wallet` functions that require a `descriptor`

`index` needs to be kept track of by the client, to avoid address reuse. Every time a user generates an address, an index counter must be incremented and every call to `get_address` must use an updated index.

### wallet/recieve

```
get_address(
  descriptor: *const c_char,
  index: *const c_char,
)->WalletAddress {
  address: String
}
```

*Only generates segwit-native addresses.*

This ends the set of functions that are better performed offline. When back online, the only thing hot is an `encrypted descriptor` representing a wallet.

### Online

#### node_address
One common parameter in all Online functions is `node_address`

This is where we pass the location of our bitcoin node.

`libstackmate` currently recommends only electrum servers for remote usage on mobile devices. libstackmate can also connect to a bitcoin node via RPC using the standard connection string format HOWEVER, this is not recommended to be done by remote mobile clients. RPC is best suited for local usage on desktop/server side applications.

### wallet/history

```
sync_balance(
  descriptor: *const c_char,
  node_address: "default" || *const c_char, ("default" or invalid *const c_char will default to blockstream server)
)->WalletBalance {
  balance: u64
}
```

```
sync_history(
  descriptor: *const c_char,
  node_address: "default" || *const c_char, ("default" or invalid *const c_char will default to blockstream server)
)->WalletHistory {
  history: Vec<Transaction {
    timestamp: u64,
    height: u32,
    verified: bool,
    txid: String,
    received: u64,
    sent: u64,
    fee: u64
   }>
}

```

### wallet/send

#### FEES

The unit for fees on the bitcoin network is `sats/byte`. 

This is a `rate` based representation of fees. 

When a wallet builds a transaction, the size of the transaction is relative to 
- the utxo set used
- all the satisfaction conditions required (signatures mainly)

Wallets end up paying an `absolute` fee to miners based on the ongoing `rate` and their transaction size.

The client must follow a specific flow to provide an intuitive experience for fees. 

First get the network rate from a node:

```
estimate_network_fee(
  network: "test" || "main", (All other strings default to "test")
  node_address: "default" || *const c_char, ("default" or invalid *const c_char will default to blockstream server)
  conf_target: *const c_char, (Values that cannot be parsed to integer will default to "6")
)->NetworkFee {
  rate: f32,
}
```

Then build a dummy transaction with a `fee_absolute` of 750 sats

The `policy_path` can always be an empty string for single sigs and `sweep` can be avoided and will default to false.

Use your intended `tx_outputs` value (this will drastically affect the size of your final transaction)
```
build_tx(
  descriptor: *const c_char,
  node_address: "default" || *const c_char, ("default" or invalid *const c_char will default to blockstream server)
  tx_outputs: *const c_char (stringified JSON array of TxOutput{address: String, amount: u64}),
  fee_absolute: *const c_char,
  policy_path: *const c_char (stringified JSON PolicyPath{id:String, path: Vec<usize>} - can be empty string or null - unparsable JSON will assume empty path)
  sweep: "true" || "false" (defaults to "false" for any other strings)
)->WalletPSBT {
  psbt: String,
  is_finalized: bool
}
```

Use the resulting `psbt` with `get_weight` to get the weight of this transaction.

```
get_weight(
  descriptor: *const c_char,
  psbt: *const c_char,
) -> TransactionWeight {
  weight: usize
}
```

Using the `weight` and various `fee_rate` options, we can get the absolute fee for this specific transaction.

```
get_absolute_fee(
  fee_rate: *const c_char,
  weight: *const c_char,
) -> NetworkFee{
  rate: f32,
  absolute: u64
}
```
Once the user is satisfied with the given `absolute` fee amount, the psbt will have to be rebuilt with the new `fee_absoulte`.

For single signature private descriptors, `build_tx` with also return an `is_finalized:true` because the wallet will automatically attempt to sign the transaction after building.


The resulting `psbt` can be used as the `signed_psbt` to be broadcasted to the bitcoin network.

```
broadcast_tx(
  descriptor: *const c_char,
  node_address: "default" || *const c_char, ("default" or invalid *const c_char will default to blockstream server)
  signed_psbt: *const c_char,
)->Txid {
  txid: String
}
```

For single sig wallets, we do not need to use `sign_tx`. It is only used for script use-cases.

```
sign_tx(
  descriptor: *const c_char,
  node_address: "default" || *const c_char, ("default" or invalid *const c_char will default to blockstream server)
  unsigned_psbt: *const c_char,
)->WalletPSBT {
  psbt: String,
  is_finalized: bool
}
```

##### DART FFI

Need to verify if dart-ffi clears ffi output pointers after use. *MOST LIKELY THE CASE*

If not, the result of your ffi function (which is a pointer) should be passed in `cstring_free` to clear memory.

```
cstring_free(ptr: *mut c_char)

```

### TOR

Provide a temp working directory for tor. Defaults to /tmp.

Returns control_key required to use tor_progress and tor_shutdown.

```
tor_start(tmp_path: *mut c_char) -> *mut c_char
```

Returns a stringidied usize between 0-100, indicating bootstrap progress.
Returns 101 incase of error. In such cases, try again (it could be too soon).

```
tor_progress(control_key: *mut c_char) -> *mut c_char 
```

Returns true or false stringified indicating successful shutdown.
```
tor_stop(control_key: *mut c_char) -> *mut c_char 
```
