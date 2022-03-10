## ffi specifications

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
derive_wallet_account(
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

```
shared_secret(
    local_secret: *const c_char,
    remote_pubkey: *const c_char,
) -> *mut c_char {
```

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
estimate_network_fee(
  network: "test" || "main", (All other strings default to "test")
  node_address: "default" || *const c_char, ("default" or invalid *const c_char will default to blockstream server)
  target_size: *const c_char, (Values that cannot be parsed to integer will default to "6")
)->NetworkFee {
  rate: f32,
  absolute: Option<u64>
}
```

```
get_weight(
  descriptor: *const c_char,
  psbt: *const c_char,
) -> TransactionWeight {
  weight: usize
}
```

```
get_absolute_fee(
  fee_rate: *const c_char,
  weight: *const c_char,
) -> NetworkFee{
  rate: f32,
  absolute: Option<u64>
}
```
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

```
get_address(
  descriptor: *const c_char,
  node_address: "default" || *const c_char, ("default" or invalid *const c_char will default to blockstream server)
  index: *const c_char,
)->WalletAddress {
  address: String
}
```

## PSBT


```
build_tx(
  descriptor: *const c_char,
  node_address: "default" || *const c_char, ("default" or invalid *const c_char will default to blockstream server)
  tx_outputs: *const c_char (stringified JSON array of TxOutput{address, amount}),
  fee_absolute: *const c_char,
  sweep: "true" || "false" (defaults to "false" for any other strings)
)->WalletPSBT {
  psbt: String,
  is_finalized: bool
}
```

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

```
broadcast_tx(
  descriptor: *const c_char,
  node_address: "default" || *const c_char, ("default" or invalid *const c_char will default to blockstream server)
  signed_psbt: *const c_char,
)->Txid {
  txid: String
}
```

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

We use only one of the following 2 script types:
- `wpkh` for segwit single sig
- `wsh` for multi-condition segwit scripts

Where conditions involve keys, the extended key format is

```
[source]xkey/*
```

Key source tells us the parent fingerprint and the hardened derived path used to reach this child key

```
[fingerprint/hardened_path]
i.e.
[fingerprint/purpose'/network'/account']
eg: [6eg88e/84h/0h/1h]
```

Where the complete derivation path in isolation is represented as 

```
m/purpose'/network'/account'/desopit/index
```
Where ' or "h" represents a hardened path &

Where m is replaced by the fingerprint in the key source and `unhardened paths m/deposit/index are set to be *` i.e. it will be rotated per payment requirements by the wallet.


A single sig policy string 

```
pk([db7d25b5/84'/1'/6']tprv8fWev2sCuSkVWYoNUUSEuqLkmmfiZaVtgxosS5jRE9fw5ejL2odsajv1QyiLrPri3ppgyta6dsFaoDVCF4ZdEAR6qqY4tnaosujsPzLxB49/*)
```

will compile into the following descriptor

```
wpkh([db7d25b5/84'/1'/6']tprv8fWev2sCuSkVWYoNUUSEuqLkmmfiZaVtgxosS5jRE9fw5ejL2odsajv1QyiLrPri3ppgyta6dsFaoDVCF4ZdEAR6qqY4tnaosujsPzLxB49/*)
```

A timlocked custodian policy string
```
or(pk([db7d25b5/84'/1'/6']tprv8fWev2sCuSkVWYoNUUSEuqLkmmfiZaVtgxosS5jRE9fw5ejL2odsajv1QyiLrPri3ppgyta6dsFaoDVCF4ZdEAR6qqY4tnaosujsPzLxB49/*),
and(pk([66a0c105/84'/1'/5']tpubDCKvnVh6U56wTSUEJGamQzdb3ByAc6gTPbjxXQqts5Bf1dBMopknipUUSmAV3UuihKPTddruSZCiqhyiYyhFWhz62SAGuC3PYmtAafUuG6R/*),after(595_600)))
```

will compile into the following descriptor

```
wsh(or_d(pk([db7d25b5/84'/1'/6']tprv8fWev2sCuSkVWYoNUUSEuqLkmmfiZaVtgxosS5jRE9fw5ejL2odsajv1QyiLrPri3ppgyta6dsFaoDVCF4ZdEAR6qqY4tnaosujsPzLxB49/*),
and_v(v:pk([66a0c105/84'/1'/5']tpubDCKvnVh6U56wTSUEJGamQzdb3ByAc6gTPbjxXQqts5Bf1dBMopknipUUSmAV3UuihKPTddruSZCiqhyiYyhFWhz62SAGuC3PYmtAafUuG6R/*),after(595600))))

```

Checkout the [Miniscript Primer](https://bitcoin.sipa.be/miniscript/) for more.

## Note on fees:

The project is currently updating build_tx to allow the use absolute fees. 

This will require using the following utils to estimate the best absolute fee to achieve your block confirmation target.

 