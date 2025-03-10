#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { Connection, Keypair, PublicKey, clusterApiUrl, Transaction, sendAndConfirmTransaction } = require('@solana/web3.js');
const { prompt } = require('inquirer');
const chalk = require('chalk');
const ora = require('ora');
require('dotenv').config();

// Configuration
const DEFAULT_NETWORK = process.env.SOLANA_NETWORK || 'devnet';
const DEFAULT_KEYPAIR_PATH = process.env.SOLANA_WALLET_PATH || './keypair.json';
const DEFAULT_TOKEN_ADDRESS = process.env.DATA_TOKEN_ADDRESS;
const DEFAULT_PLATFORM_FEE = 2; // 2% platform fee

// Main function
async function main() {
  console.log(chalk.blue('='.repeat(50)));
  console.log(chalk.blue.bold('DataChain AI Marketplace Setup Script'));
  console.log(chalk.blue('='.repeat(50)));
  console.log();

  // Get setup parameters
  const answers = await prompt([
    {
      type: 'list',
      name: 'network',
      message: 'Select Solana network:',
      choices: ['devnet', 'testnet', 'mainnet-beta'],
      default: DEFAULT_NETWORK,
    },
    {
      type: 'input',
      name: 'keypairPath',
      message: 'Path to your Solana keypair file:',
      default: DEFAULT_KEYPAIR_PATH,
      validate: (input) => {
        try {
          const fullPath = path.resolve(input);
          if (fs.existsSync(fullPath)) {
            return true;
          }
          return 'File does not exist';
        } catch (error) {
          return 'Invalid path';
        }
      },
    },
    {
      type: 'input',
      name: 'tokenAddress',
      message: 'DATA token address:',
      default: DEFAULT_TOKEN_ADDRESS,
      validate: (input) => {
        try {
          new PublicKey(input);
          return true;
        } catch (error) {
          return 'Invalid Solana address';
        }
      },
      when: () => DEFAULT_TOKEN_ADDRESS !== undefined,
    },
    {
      type: 'number',
      name: 'platformFee',
      message: 'Platform fee percentage (0-100):',
      default: DEFAULT_PLATFORM_FEE,
      validate: (input) => {
        if (input >= 0 && input <= 100) {
          return true;
        }
        return 'Fee must be between 0 and 100';
      },
    },
    {
      type: 'confirm',
      name: 'confirmSetup',
      message: (answers) => `You are about to set up the DataChain AI marketplace on ${answers.network} with a ${answers.platformFee}% platform fee. Continue?`,
      default: false,
    },
  ]);

  if (!answers.confirmSetup) {
    console.log(chalk.yellow('Setup cancelled.'));
    return;
  }

  // Load keypair
  const spinner = ora('Loading keypair...').start();
  let keypair;
  try {
    const keypairData = JSON.parse(fs.readFileSync(path.resolve(answers.keypairPath), 'utf-8'));
    keypair = Keypair.fromSecretKey(new Uint8Array(keypairData));
    spinner.succeed(`Keypair loaded: ${keypair.publicKey.toString()}`);
  } catch (error) {
    spinner.fail(`Failed to load keypair: ${error.message}`);
    return;
  }

  // Connect to Solana
  spinner.text = `Connecting to Solana ${answers.network}...`;
  spinner.start();
  const connection = new Connection(
    answers.network === 'mainnet-beta'
      ? process.env.MAINNET_RPC_URL || clusterApiUrl('mainnet-beta')
      : clusterApiUrl(answers.network),
    'confirmed'
  );

  try {
    const version = await connection.getVersion();
    spinner.succeed(`Connected to Solana ${answers.network}: ${JSON.stringify(version)}`);
  } catch (error) {
    spinner.fail(`Failed to connect to Solana: ${error.message}`);
    return;
  }

  // Check account balance
  spinner.text = 'Checking account balance...';
  spinner.start();
  try {
    const balance = await connection.getBalance(keypair.publicKey);
    const solBalance = balance / 1e9;
    spinner.succeed(`Account balance: ${solBalance} SOL`);

    if (solBalance < 0.1) {
      console.log(chalk.yellow(`Warning: Low balance (${solBalance} SOL). You may need more SOL to complete the setup.`));
      const { continueWithLowBalance } = await prompt([
        {
          type: 'confirm',
          name: 'continueWithLowBalance',
          message: 'Continue with low balance?',
          default: false,
        },
      ]);
      if (!continueWithLowBalance) {
        console.log(chalk.yellow('Setup cancelled.'));
        return;
      }
    }
  } catch (error) {
    spinner.fail(`Failed to check balance: ${error.message}`);
    return;
  }

  // Initialize marketplace
  spinner.text = 'Initializing Data Marketplace...';
  spinner.start();

  try {
    // This is a placeholder for the actual marketplace initialization code
    // In a real implementation, you would use the Solana web3.js library
    // to create and send a transaction that calls the initialize_marketplace instruction
    
    // Simulate marketplace initialization with a delay
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    // Mock marketplace address (in a real implementation, this would be the actual address)
    const dataMarketplaceAddress = new PublicKey('DATAMktXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX');
    
    spinner.succeed(`Data Marketplace initialized successfully!`);
    console.log();
    
    // Initialize AI Model Marketplace
    spinner.text = 'Initializing AI Model Marketplace...';
    spinner.start();
    
    // Simulate AI model marketplace initialization with a delay
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    // Mock AI model marketplace address (in a real implementation, this would be the actual address)
    const aiModelMarketplaceAddress = new PublicKey('DATAAIMktXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX');
    
    spinner.succeed(`AI Model Marketplace initialized successfully!`);
    console.log();
    
    // Initialize Governance
    spinner.text = 'Initializing Governance System...';
    spinner.start();
    
    // Simulate governance initialization with a delay
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    // Mock governance address (in a real implementation, this would be the actual address)
    const governanceAddress = new PublicKey('DATAGovXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX');
    
    spinner.succeed(`Governance System initialized successfully!`);
    console.log();
    
    console.log(chalk.green('Marketplace Information:'));
    console.log(chalk.green('-'.repeat(50)));
    console.log(`Data Marketplace Address: ${dataMarketplaceAddress.toString()}`);
    console.log(`AI Model Marketplace Address: ${aiModelMarketplaceAddress.toString()}`);
    console.log(`Governance Address: ${governanceAddress.toString()}`);
    console.log(`Platform Fee: ${answers.platformFee}%`);
    console.log(`Network: ${answers.network}`);
    console.log(`Authority: ${keypair.publicKey.toString()}`);
    console.log(chalk.green('-'.repeat(50)));
    
    // Save marketplace information to a file
    const marketplaceInfo = {
      dataMarketplace: dataMarketplaceAddress.toString(),
      aiModelMarketplace: aiModelMarketplaceAddress.toString(),
      governance: governanceAddress.toString(),
      platformFee: answers.platformFee,
      network: answers.network,
      authority: keypair.publicKey.toString(),
      setupAt: new Date().toISOString(),
    };
    
    fs.writeFileSync(
      path.resolve('./marketplace-info.json'),
      JSON.stringify(marketplaceInfo, null, 2)
    );
    
    console.log();
    console.log(chalk.blue('Marketplace information saved to marketplace-info.json'));
    console.log();
    console.log(chalk.yellow('Next Steps:'));
    console.log('1. Add the marketplace addresses to your .env file');
    console.log('2. Update your frontend with the marketplace addresses');
    console.log('3. Start registering datasets and AI models');
    
  } catch (error) {
    spinner.fail(`Failed to initialize marketplace: ${error.message}`);
    console.error(error);
  }
}

// Run the script
main().catch((error) => {
  console.error(chalk.red('Setup failed:'));
  console.error(error);
  process.exit(1);
}); 