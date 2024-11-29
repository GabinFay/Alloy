const ethers = require('ethers');
const fs = require('fs-extra');
require('dotenv').config();

// Load environment variables
const PROVIDER_URL = process.env.PROVIDER_URL;

// Contract addresses
const WETH_ADDRESS = '0x4200000000000000000000000000000000000006';

// ABI for the token contract
const TOKEN_ABI = [
  'event Transfer(address indexed from, address indexed to, uint256 value)'
];

let provider;

const initializeProvider = async () => {
  try {
    console.log('Initializing provider...');
    provider = new ethers.providers.WebSocketProvider(PROVIDER_URL);

    provider._websocket.on('open', () => {
      console.log('WebSocket connection established successfully.');
    });
    monitorWethTransfers();
  } catch (error) {
    console.error('Failed to initialize provider:', error);
  }
};

// Monitor Transfer events for virtual token to track balance changes
const monitorWethTransfers = () => {
  try {
    console.log('Starting to monitor Transfer events for WETH...');
    const wethContract = new ethers.Contract(WETH_ADDRESS, TOKEN_ABI, provider);
    wethContract.on('Transfer', handleTransferEvent);
    console.log('Monitoring of Transfer events started successfully.');
  } catch (error) {
    console.error('Error starting to monitor Transfer events:', error);
  }
};

const handleTransferEvent = async (from, to, value) => {
  try {
    const amount = ethers.utils.formatUnits(value, 18);
    console.log(`Transfer event detected. From: ${from}, To: ${to}, Amount: ${amount}`);
  } catch (error) {
    console.error('Error handling Transfer event:', error);
  }
};

// Initialize provider on startup
console.log('Starting initialization of provider...');
initializeProvider();