# api
### stackmate-bdk ffi documentation

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

```
derive_hardened(
    master_xprv: *const c_char,
    purpose: "84" || "49" || "44", (All other strings default to "84")
    account: *const c_char, (Can be empty - will default to "0" if value cannot be parsed to integer)
)->ChildKeys {
  fingerprint: String,
  hardened_path: String,
  xprv: String,
  xpub: String
}
```

```
compile(
  policy: *const c_char, 
  script_type: "wpkh" || "wsh" || "sh" || "pk", (Defaults to "wpkh" for all others)
)->WalletPolicy {
  policy: String,
  descriptor: String
}

```

```
get_fees(
  network: "test" || "main", (All other strings default to "test")
  node_address: "default" || *const c_char, ("default" or invalid *const c_char will default to blockstream server)
  target_size: *const c_char, (Values that cannot be parsed to integer will default to "6")
)->NetworkFee {
  fee: f32
}
```

```
sync_balance(
  deposit_desc: *const c_char,
  node_address: "default" || *const c_char, ("default" or invalid *const c_char will default to blockstream server)
)->WalletBalance {
  balance: u64
}
```

```
sync_history(
  deposit_desc: *const c_char,
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

```
get_address(
  deposit_desc: *const c_char,
  node_address: "default" || *const c_char, ("default" or invalid *const c_char will default to blockstream server)
  index: *const c_char,
)->WalletAddress {
  address: String
}
```

## build_tx



```
build_tx(
  deposit_desc: *const c_char,
  node_address: "default" || *const c_char, ("default" or invalid *const c_char will default to blockstream server)
  to_address: *const c_char,
  amount: *const c_char, (Use "0" when combined with sweep)
  fee_rate: *const c_char,
  sweep: "true" || "false" (defaults to "false" for any other strings)
)->WalletPSBT {
  psbt: String,
  is_finalized: bool
}
```

```
sign_tx(
  deposit_desc: *const c_char,
  node_address: "default" || *const c_char, ("default" or invalid *const c_char will default to blockstream server)
  unsigned_psbt: *const c_char,
)->WalletPSBT {
  psbt: String,
  is_finalized: bool
}
```

```
broadcast_tx(
  deposit_desc: *const c_char,
  node_address: "default" || *const c_char, ("default" or invalid *const c_char will default to blockstream server)
  signed_psbt: *const c_char,
)->Txid {
  txid: String
}
```

```
cstring_free(ptr: *mut c_char)

```

```
Error Format:

S5Error {
  kind: ErrorKind {
    InputError, // error in input params
    OpError, // internal error
  },
  message: String,
}

```

## Note on descriptors:

The descriptor format is
```
script(conditions)
```

Where conditions involve keys, the extended key format is

```
[fingerprint/hardened_path]key/unhardened_path
```

Making the format for a deposit descriptor as

```
[fingerprint/purpose'/network'/account']key/0/*
```


And the format for a change descriptor as (done internally)

```
[fingerprint/purpose'/network'/account']key/1/*
```

Where the complete derivation path in isolation is represented as 

```
m/purpose'/network'/account'/desopit/index
```
Where ' or "h" represents a hardened path &

Where m is replaced by the fingerprint in the extended key format and unhardened paths deposit/index follows the key.

## Note on fees:

The project is currently updating build_tx to allow the use absolute fees. 

This will require using the following utils to estimate the best absolute fee to achieve your block confirmation target.

 