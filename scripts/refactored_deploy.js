import { LCDClient, MnemonicKey, MsgExecuteContract } from '@terra-money/terra.js';


import { storeCode, instantiateContract, sendTx, queueAndToggle } from './helpers.js';
// import { send } from 'process';

// test1 key from localterra accounts
const mk = new MnemonicKey({
//   mnemonic: 'satisfy adjust timber high purchase tuition stool faith fine install that you unaware feed domain license impose boss human eager hat rent enjoy dawn'
    mnemonic: 'spatial forest elevator battle also spoon fun skirt flight initial nasty transfer glory palm drama gossip remove fan joke shove label dune debate quick'
})

// connect to testnet
// const terra = new LCDClient({
//     URL: 'https://bombay-lcd.terra.dev',
//     chainID: 'bombay-12',
// });

//connect to localterra
const terra = new LCDClient({
    URL: "http://localhost:1317",
    chainID: "localterra",
});


const wallet = terra.wallet(mk);


let contractAddress = "";

console.log("Big Bang");



/* TREASURY INSTANTIATION */
let contractCodeId = await storeCode(wallet, terra, "../artifacts/interview_task.wasm");
const contractInstantiateMsg = {
    company:  "terra1dcegyrekltswvyy0xy69ydgxn9x8x32zdtapd8",
    user: wallet.key.accAddress,

    
};
contractAddress = await instantiateContract(wallet , terra, contractCodeId, contractInstantiateMsg);
console.log("CONTRACT_ADDRESS: " + '"' + contractAddress + '"' + ',');



