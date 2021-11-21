import { HashType } from '@ckb-lumos/base';
import { HexString } from '@ckb-lumos/base';
import { SellOptions, BuyOptions, ListSellingResponse } from '../selling-lock.d'

import { SCRIPTS } from '../config.json';
const { SUDT } = SCRIPTS;

export async function sell(_options: SellOptions, privKey: HexString) {
  return "0x12345"
}

export async function buy(_options: BuyOptions, privKey: HexString) {
  return "0x12345"
}

export async function listSelling(): Promise<ListSellingResponse[]> {
  const response: ListSellingResponse[] = [{
    seller: 'ckt1qyqxn59qn8xx4zd8p6mu4xtf4lz94zf2vfwsjaffax',
    sudt: {
      code_hash: SUDT.CODE_HASH,
      hash_type: SUDT.HASH_TYPE as HashType,
      args: "0x"
    },
    amount: "0x02",
    sellPrice: "0x01",
    selling: {
      tx_hash: "b0d883a7ad457324396dd68befba5cd8af5b8722f1f6b0a7cd8347bafaff1956",
      index: "0x00"
    }
  }]
  return Promise.resolve(response)
}
