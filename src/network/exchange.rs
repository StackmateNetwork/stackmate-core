const SATS_DENOMINATOR = 100000000.0;

pub enum FiatUnit{
  INR,
  CAD,
  USD,
  EUR,
}

pub enum SourceExchange{
  Custom,
  LocalBitcoins,
  BullBitcoin,
  Strike,
  Kraken
}

/// btc contains value of 1 BTC as FiatUnit
/// sats contains value of 1 FiatUnit as sats
pub struct Rate {
  timestamp: u64,
  fiat_unit: FiatUnit
  source: SourceExchange,
  btc: f64,
  sats: f64
}

impl Rate{
  pub fn now(source: Exchange) -> Rate {
    // TODO: implement with source exchanges
    Rate {
      timestamp: timestamp,
      fiat_unit: FiatUnit::INR,
      source,
      btc: 55000000.0,
      sats: 18.2
    }
  }
  pub fn from_btc_value(fiat_unit: FiatUnit, btc_value: f64, fiat_value: f64) -> Rate{
    Rate {
      timestamp: timestamp,
      fiat_unit: fiat_unit,
      source: SourceExchange::Custom,
      btc: fiat_value/btc_value,
      sats: btc_value*SATS_DENOMINATOR/fiat_value
    }
  }
  pub fn from_sats_value(fiat_unit: FiatUnit, sats_value: f64, fiat_value: f64) -> Rate{
    Rate {
      timestamp: timestamp,
      fiat_unit: fiat_unit,
      source: SourceExchange::Custom,
      btc: fiat_value/( sats_value/SATS_DENOMINATOR ),
      sats: sats_value/fiat_value
    }
  }
  pub fn add(&self, percentage: usize) -> Rate{
    Rate {
      timestamp: self.timestamp,
      fiat_unit: self.fiat_unit,
      source: SourceExchange::Custom,
      btc: self.btc + (self.btc * percentage as f64 / 100.0),
      sats: self.sats + (self.sats * percentage as f64 / 100.0)
    }
  }
  pub fn sub(&self, percentage: usize) -> Rate{
    Rate {
      timestamp: self.timestamp,
      fiat_unit: self.fiat_unit,
      source: SourceExchange::Custom,
      btc: self.btc - (self.btc * percentage as f64 / 100.0),
      sats: self.sats - (self.sats * percentage as f64 / 100.0)
    }
  }
}
