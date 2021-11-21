import React, { useState } from "react";
import { sell } from "../apis/mock";
import { SellOptions } from "../selling-lock";
import { generateSecp256k1Blake160Address } from "@ckb-lumos/helpers"
const config = require("../config.json");

interface SellForm {
  sellerPrivKey: string;
  sudtCodeHash: string;
  amount: number;
  sellPrice: number;
}

const initialSellForm: SellForm = {
  sellerPrivKey: "",
  sudtCodeHash: "",
  amount: 0,
  sellPrice: 0
}

export default function SellCells() {
  const [form, setForm] = useState<SellForm>(initialSellForm);
  const [message, setMessage] = useState<string>('');

  const sellCell = async () => {
    if(form.sellerPrivKey && form.sudtCodeHash && form.amount && form.sellPrice){
      let options: SellOptions = {
        seller: generateSecp256k1Blake160Address(form.sellerPrivKey, {config}),
        sudt: {
          code_hash: form.sudtCodeHash,
          hash_type: "type",
          args: "0x"
        },
        amount: `0x${BigInt(form.amount).toString(16)}`,
        sellPrice: `0x${BigInt(form.sellPrice).toString(16)}`
      }
      let txHash = await sell(options, form.sellerPrivKey);
      setMessage(`Sell Success, tx hash is: ${txHash}`)
    } else {
      setMessage('Please fill in all fields')
    }
  };

  const formItemClass = "mb-2 bg-green-700 bg-opacity-90 h-16 rounded-md shadow-lg border-2 border-solid border-gray-800 p-4"
  return (
    <div className="">
      <p className="mt-8 h-16 text-3xl flex flex-row justify-center text-gray-200 items-center bg-green-900 bg-opacity-90">
        Selling Your USDT
      </p>
      <div className="bg-green-700 bg-opacity-90 p-8">
        <div className={formItemClass}>
          private key: <input className="inline-block w-10/12 bg-gray-900 bg-opacity-40 text-gray-300" value={form.sellerPrivKey} onChange={(e) => setForm({...form, sellerPrivKey: e.target.value})}/>
        </div>
        <div className={formItemClass}>
          code hash: <input className="inline-block w-10/12 bg-gray-900 bg-opacity-40 text-gray-300" value={form.sudtCodeHash} onChange={(e) => setForm({...form, sudtCodeHash: e.target.value})}/>
        </div>
        <div className={formItemClass}>
         sell amount: <input type="number" className="inline-block w-10/12 bg-gray-900 bg-opacity-40 text-gray-300" value={form.amount} onChange={(e) => setForm({...form, amount: e.target.value as any})}/>
        </div>
        <div className={formItemClass}>
          sell price: <input type="number" className="inline-block w-10/12 bg-gray-900 bg-opacity-40 text-gray-300" value={form.sellPrice} onChange={(e) => setForm({...form, sellPrice: e.target.value as any})}/>
        </div>
        <div className="mb-2 h-1 p-4">
          <button className="button hover:bg-gray-700 hover:text-gray-200 border-2 border-solid border-gray-800 px-2 rounded-md mt-2" onClick={(() => sellCell() ) as any}> 购买 </button>
          <span className="ml-2">{message}</span>
        </div>
      </div>
     
         
      {/* <div className="flex flex-row flex-wrap justify-between bg-green-700 bg-opacity-90">
        {list.map((item: ListSellingResponse, index: number) => {
          return <div className="h-48 w-132 m-8 rounded-md shadow-lg border-2 border-solid border-gray-800 p-4" key={index}>
            <div className="name">卖家：{item.seller}</div>
            <div className="truncate">sudt：{item.sudt.code_hash}</div>
            <div className="price">价格：{item.sellPrice}</div>
            <div className="amount">数量：{item.amount}</div>
            <button className="button hover:bg-gray-700 hover:text-gray-200 border-2 border-solid border-gray-800 px-2 rounded-md mt-2" onClick={(() => buyCell(item) ) as any}> 购买 </button>
            <span className="ml-2">{message}</span>
          </div>;
        })}
      </div> */}
    </div>
  );
}
