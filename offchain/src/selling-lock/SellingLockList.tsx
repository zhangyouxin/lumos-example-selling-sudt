import React, { useEffect, useState } from "react";
import { ListSellingResponse } from "../selling-lock";
import { listSelling, buy } from "../apis/mock";
import { BuyOptions } from "../selling-lock.d";
import { generateSecp256k1Blake160Address } from "@ckb-lumos/helpers"
const config = require("../config.json");
console.log(config)

export default function SellingLockList(props:{ privKey: string, setPrivKey: any }) {
  const [list, setList] = useState<ListSellingResponse[]>([]);
  const [message, setMessage] = useState<string>('');

  useEffect(() => {
    retrieveSellingLocks();
  }, []);

  const retrieveSellingLocks = () => {
    listSelling().then((items: ListSellingResponse[]) => {
      setList(items);
    });
  };

  const buyCell = async (item: ListSellingResponse) => {
    if(!props.privKey){
      setMessage("Please fill in buyer privKey first.");
    } else {
      const buyer = generateSecp256k1Blake160Address(props.privKey, {config});
      const buyOptions: BuyOptions = {
        selling: item.selling,
        sellPrice: item.sellPrice,
        buyer
      }
      let hash = await buy(buyOptions, props.privKey);
      setMessage(`Buy success! buyer is ${buyer}, tx hash is: ${hash}`);
    }
  }

  return (
    <div className="">
      <p className="h-16 text-3xl flex flex-row justify-center text-gray-200 items-center bg-green-900 bg-opacity-90">
        List Selling Locks
      </p>
      <div className="bg-green-900 bg-opacity-90 p-2">
          buyer private key: <input className="inline-block w-96 bg-gray-900 bg-opacity-40 text-gray-300" value={props.privKey} onChange={(e) => props.setPrivKey(e.target.value)}/>
        <button className="ml-4 button hover:bg-gray-700 hover:text-gray-200 border-2 border-solid border-gray-800 px-2 rounded-md mt-2" onClick={(() => retrieveSellingLocks() ) as any}> 刷新 </button>
      </div>
      <div className="flex flex-row flex-wrap justify-between bg-green-700 bg-opacity-90" style={{ minHeight: '12rem'}}>
        {list.map((item: ListSellingResponse, index: number) => {
          return <div className="h-48 w-132 m-8 rounded-md shadow-lg border-2 border-solid border-gray-800 p-4" key={index}>
            <div className="name">卖家：{item.seller}</div>
            <div className="truncate">sudt：{item.sudt.code_hash}</div>
            <div className="price">价格：{BigInt(item.sellPrice).toString()}</div>
            <div className="amount">数量：{BigInt(item.amount).toString()}</div>
            <button className="button hover:bg-gray-700 hover:text-gray-200 border-2 border-solid border-gray-800 px-2 rounded-md mt-2" onClick={(() => buyCell(item) ) as any}> 购买 </button>
            <span className="ml-2">{message}</span>
          </div>;
        })}
      </div>
    </div>
  );
}
