import React from 'react';
import SellingLockList from './SellingLockList';
import SellCells from './SellCells';

function App() {
  const [privKey, setPrivKey] = React.useState<string>('0xb12c5d587aef846f2e6031d5c73d169db7e434441cfd11c185207d6b8b71aabb');

  return (
    <div className="min-h-screen bg-gray-700 bg-opacity-0.9">
      <header className="h-16 text-3xl flex flex-row justify-center text-gray-200 items-center bg-gray-500">
        Selling Lock Demo
      </header>
      <div className="m-4 p-4 flex flex-row">
        <div className="flex-grow">
          <SellingLockList  privKey={privKey} setPrivKey={setPrivKey} />
          <SellCells />
        </div>
      </div>
    </div>
  );
}

export default App;
