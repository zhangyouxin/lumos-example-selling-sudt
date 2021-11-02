import { Address, HexNumber, OutPoint, Script } from '@ckb-lumos/base';
import { TransactionSkeletonType } from '@ckb-lumos/helpers';

function unimplemented(): never {
  throw new Error('unimplemented');
}

type TypeScript = Script;
type LockScript = Script;

interface SellOptions {
  seller: Address;
  sudt: TypeScript;
  amount: HexNumber;
  // selling price, shannon, the basic unit of CKB, 1 CKB = (10 ^ -8) shannon)
  sellPrice: HexNumber;
}

function sell(_options: SellOptions): Promise<TransactionSkeletonType> {
  unimplemented();
}

interface BuyOptions {
  // an out point will actually point to a specific cell
  selling: OutPoint;
  sellPrice: HexNumber;
  buyer: Address;
}

function buy(_options: BuyOptions): Promise<TransactionSkeletonType> {
  unimplemented();
}

interface ListSellingResponse {
  seller: Address;
  sudt: TypeScript;
  amount: HexNumber;
  sellPrice: HexNumber;
  selling: OutPoint;
}

function listSelling(): Promise<ListSellingResponse[]> {
  unimplemented();
}