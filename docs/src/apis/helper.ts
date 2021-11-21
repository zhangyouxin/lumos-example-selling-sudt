import { HashType } from '@ckb-lumos/base';
import { SCRIPTS } from '../config.json';
import { Script, Cell, CellProvider, DepType, utils, HexString, Transaction } from '@ckb-lumos/base';
import {
  TransactionSkeletonType,
  parseAddress,
  sealTransaction,
  TransactionSkeleton,
} from '@ckb-lumos/helpers';
import { Indexer, CellCollector } from "@ckb-lumos/ckb-indexer"
import { common } from "@ckb-lumos/common-scripts"
import { key } from "@ckb-lumos/hd";
import { BuyOptions, SellOptions } from '../selling-lock';
import { RPC } from "@ckb-lumos/rpc";
const { SELLING_LOCK, SUDT } = SCRIPTS;

const CONFIG = require('../config.json');
const FEE = BigInt(10000000);
const SELL_CELL_CAPACITY = BigInt(200) * BigInt(100000000);
// const INDEXER = new Indexer("http://127.0.0.1:9116", "http://127.0.0.1:9114");
// const ckb = new RPC("http://127.0.0.1:9114");

const ckb = new RPC("https://testnet.ckb.dev");
const INDEXER = new Indexer("https://testnet.ckb.dev/indexer", "https://testnet.ckb.dev");

export async function listSellingCells() {
  let cellCollector = new CellCollector(INDEXER, {
    lock: {
      code_hash: SELLING_LOCK.CODE_HASH,
      hash_type: SELLING_LOCK.HASH_TYPE as HashType,
      args: "0x" // search selling lock with any args
    },
  })
  let cells: Cell[] = []

  for await (const cell of cellCollector.collect()) {
    cells.push(cell as Cell)
  }

  return cells
}

export async function buildBuyTx(_options: BuyOptions): Promise<TransactionSkeletonType> {
  let fromAddress = _options.buyer;
  let fromLock = parseAddress(fromAddress)
  let skeleton = TransactionSkeleton({ cellProvider: INDEXER as CellProvider });

  await ckb.get_transaction(_options.selling.tx_hash).then(async (tx) => {
    if (!tx) {
      throw new Error("Transaction not found");
    }
    const sellingCellOutput = tx.transaction.outputs[Number(_options.selling.index)];
    const sellingCellData = tx.transaction.outputs_data[Number(_options.selling.index)];
    const sellingCell: Cell = {
      cell_output: sellingCellOutput,
      data: sellingCellData
    }
    const args = sellingCell.cell_output.lock.args;
    skeleton = skeleton.update("inputs", (inputs) => {
      return inputs.push(sellingCell);
    });

    const buyerCell: Cell = {
      cell_output: {
        capacity: sellingCell.cell_output.capacity,
        lock: fromLock,
        type: sellingCell.cell_output.type
      },
      data: sellingCellData
    }
    skeleton = skeleton.update("outputs", (inputs) => {
      return inputs.push(buyerCell);
    });
    const payingCell: Cell = {
      cell_output: {
        capacity: _options.sellPrice,
        lock: buildLockFromArgs(args).lock,
      },
      data: "0x"
    }
    skeleton = skeleton.update("outputs", (inputs) => {
      return inputs.push(payingCell);
    });
  })
  skeleton = addCellDeps(skeleton);
  skeleton = fixInputsAndOutputs(skeleton, 1, 2);
  skeleton = await common.injectCapacity(skeleton, [fromAddress], SELL_CELL_CAPACITY + BigInt(_options.sellPrice));
  skeleton = await common.payFee(skeleton, [fromAddress], FEE);
  return skeleton
}

export async function buildSellTx(_options: SellOptions): Promise<TransactionSkeletonType> {
  let fromAddress = _options.seller;
  let fromLock = parseAddress(fromAddress)
  let skeleton = TransactionSkeleton({ cellProvider: INDEXER as CellProvider });

  let cellCollector = new CellCollector(INDEXER, {
    lock: fromLock,
    type: _options.sudt,
  })
  let cellsToSpend: Array<Cell> = []

  for await (const cell of cellCollector.collect()) {
    cellsToSpend.push(cell as Cell)
  }
  let sudtBalance = cellsToSpend.reduce((acc, cell) => acc + BigInt(cell.data), BigInt(0))
  if (sudtBalance < BigInt(_options.amount)) {
    throw new Error(`Not enough sudt balance: ${sudtBalance} < ${_options.amount}`)
  }

  console.log("sudtBalance is:", sudtBalance);
  console.log("cellToSpend is:", cellsToSpend);
  console.log("getconfig is:", CONFIG);

  skeleton = addCellDeps(skeleton);
  skeleton = skeleton.update("inputs", (inputs) => {
    return inputs.concat(cellsToSpend);
  });

  const price = _options.sellPrice;
  console.log("price is:", utils.readBigUInt64LE(price));
  const sellingArg = `0x${fromLock.code_hash.slice(2)}00${fromLock.args.slice(2)}${price.slice(2)}`
  console.log("sellingArg is:", sellingArg);
  let aliceSellingLock: Script = {
    code_hash: SELLING_LOCK.CODE_HASH,
    hash_type: "data",
    args: sellingArg,
  }
  skeleton = skeleton.update("outputs", (outputs) => {
    return outputs.push({
      cell_output: {
        capacity: "0x" + (BigInt(200) * BigInt(100000000)).toString(16),
        lock: aliceSellingLock,
        type: _options.sudt,
      },
      data: _options.amount,
    });
  });
  skeleton = fixInputsAndOutputs(skeleton, cellsToSpend.length, 1);
  skeleton = await common.injectCapacity(skeleton, [fromAddress], SELL_CELL_CAPACITY);
  skeleton = await common.payFee(skeleton, [fromAddress], FEE);
  return skeleton
}

export function signAndSeal(skeleton: TransactionSkeletonType, privKey: HexString): Transaction {
  skeleton = common.prepareSigningEntries(skeleton);
  const message = skeleton.get("signingEntries").get(0)!.message;
  console.log("message is:", message);
  const Sig = key.signRecoverable(message!, privKey);
  const tx = sealTransaction(skeleton, [Sig]);
  console.log(tx);
  return tx
}

export function buildLockFromArgs(args: HexString): { lock: Script, price: BigInt } {
  let argslen = args.length;
  let codeHash = args.slice(2, 66);
  let hashType = BigInt(args.slice(66, 68));
  let lockArgs = args.slice(68, argslen - 16);
  let price = BigInt(args.slice(argslen - 16, argslen));
  return {
    lock: {
      code_hash: codeHash,
      hash_type: hashType === BigInt(0) ? "data" : "type",
      args: lockArgs,
    },
    price
  }
}

function addCellDeps(skeleton: TransactionSkeletonType): TransactionSkeletonType {
  // add secp256k1 lock script dep
  skeleton = skeleton.update("cellDeps", (cellDeps) => {
    return cellDeps.push({
      out_point: {
        tx_hash: CONFIG.SCRIPTS!.SECP256K1_BLAKE160!.TX_HASH,
        index: CONFIG.SCRIPTS!.SECP256K1_BLAKE160!.INDEX,
      },
      dep_type: CONFIG.SCRIPTS!.SECP256K1_BLAKE160!.DEP_TYPE,
    });
  });
  // add selling lock script dep
  skeleton = skeleton.update("cellDeps", (cellDeps) => {
    return cellDeps.push({
      out_point: {
        tx_hash: SELLING_LOCK.TX_HASH,
        index: SELLING_LOCK.INDEX,
      },
      dep_type: SELLING_LOCK.DEP_TYPE as DepType,
    });
  });
  // add sudt script dep
  skeleton = skeleton.update("cellDeps", (cellDeps) => {
    return cellDeps.push({
      out_point: {
        tx_hash: SUDT.TX_HASH,
        index: SUDT.INDEX,
      },
      dep_type: SUDT.DEP_TYPE as DepType,
    });
  });
  return skeleton
}

function fixInputsAndOutputs(skeleton: TransactionSkeletonType, inputsCount: number, outputsCount: number): TransactionSkeletonType {
  skeleton = skeleton.update("fixedEntries", (fixedEntries) => {
    return fixedEntries.concat(Array(inputsCount).fill(0).map((_, index) => ({ field: "inputs", index })));
  });
  skeleton = skeleton.update("fixedEntries", (fixedEntries) => {
    return fixedEntries.concat(Array(outputsCount).fill(0).map((_, index) => ({ field: "outputs", index })));
  });
  return skeleton
}