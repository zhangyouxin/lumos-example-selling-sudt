import { HashType } from '@ckb-lumos/base';
import { OutPoint, Cell, HexString, utils } from '@ckb-lumos/base';
import { scriptToAddress } from '@ckb-lumos/helpers';
import { SellOptions, BuyOptions, ListSellingResponse } from '../selling-lock.d'
import { buildBuyTx, buildLockFromArgs, buildSellTx, listSellingCells, signAndSeal } from './helper';
import { RPC } from "@ckb-lumos/rpc";

import { SCRIPTS } from '../config.json';
const { SUDT } = SCRIPTS;
const ckb = new RPC("https://testnet.ckb.dev");

export async function sell(_options: SellOptions, privKey: HexString) {
  let skeleton = await buildSellTx(_options);
  let tx = signAndSeal(skeleton, privKey);
  let hash = ckb.send_transaction(tx);
  return hash
}

export async function buy(_options: BuyOptions, privKey: HexString) {
  let skeleton = await buildBuyTx(_options);
  let tx = signAndSeal(skeleton, privKey);
  let hash = ckb.send_transaction(tx);
  return hash
}

export async function listSelling(): Promise<ListSellingResponse[]> {
  const cells: Cell[] = await listSellingCells();
  const response: ListSellingResponse[] = cells.map(cell => {
    const { lock: ownerLock, price } = buildLockFromArgs(cell.cell_output.lock.args);
    return {
      seller: scriptToAddress(ownerLock),
      sudt: { code_hash: SUDT.CODE_HASH, hash_type: SUDT.HASH_TYPE as HashType, args: "0x" },
      amount: cell.data,
      sellPrice: utils.toBigUInt64LE(price as bigint),
      selling: cell.out_point as OutPoint,
    }
  })
  return Promise.resolve(response)
}
