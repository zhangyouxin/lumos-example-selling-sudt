import { Address, HexNumber, OutPoint, Script } from '@ckb-lumos/base';

type TypeScript = Script;
type LockScript = Script;

interface SellOptions {
  seller: Address;
  sudt: TypeScript;
  amount: HexNumber;
  // selling price, shannon, the basic unit of CKB, 1 CKB = (10 ^ -8) shannon)
  sellPrice: HexNumber;
}

interface BuyOptions {
  // an out point will actually point to a specific cell
  selling: OutPoint;
  sellPrice: HexNumber;
  buyer: Address;
}

interface ListSellingResponse {
  seller: Address;
  sudt: TypeScript;
  amount: HexNumber;
  sellPrice: HexNumber;
  selling: OutPoint;
}

interface LiveCellResponse {
  seller: Address;
  sudt: TypeScript;
  amount: HexNumber;
  sellPrice: HexNumber;
  selling: OutPoint;
}
