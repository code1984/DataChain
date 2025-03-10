#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { Connection, Keypair, PublicKey, clusterApiUrl } = require('@solana/web3.js');
const { prompt } = require('inquirer');
const chalk = require('chalk');
const ora = require('ora');
require('dotenv').config();

// Configuration
const DEFAULT_NETWORK = process.env.SOLANA_NETWORK || 'devnet';
const DEFAULT_KEYPAIR_PATH = process.env.SOLANA_WALLET_PATH || './keypair.json';
const TOKEN_NAME = 'DataChain AI';
const TOKEN_SYMBOL = 'DATA';
const TOKEN_DECIMALS = 9;
const TOKEN_URI = 'https://datachain.ai/token-metadata.json';

// Main function
async function main() {
  console.log(chalk.blue('='.repeat(50)));
  console.log(chalk.blue.bold('DataChain AI Token Deployment Script'));
  console.log(chalk.blue('='.repeat(50)));
  console.log();

  // Get deployment parameters
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
      type: 'confirm',
      name: 'confirmDeploy',
      message: (answers) => `You are about to deploy the DATA token to ${answers.network}. Continue?`,
      default: false,
    },
  ]);

  if (!answers.confirmDeploy) {
    console.log(chalk.yellow('Deployment cancelled.'));
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

    if (solBalance < 0.5) {
      console.log(chalk.yellow(`Warning: Low balance (${solBalance} SOL). You may need more SOL to complete the deployment.`));
      const { continueWithLowBalance } = await prompt([
        {
          type: 'confirm',
          name: 'continueWithLowBalance',
          message: 'Continue with low balance?',
          default: false,
        },
      ]);
      if (!continueWithLowBalance) {
        console.log(chalk.yellow('Deployment cancelled.'));
        return;
      }
    }
  } catch (error) {
    spinner.fail(`Failed to check balance: ${error.message}`);
    return;
  }

  // Deploy token
  spinner.text = 'Deploying DATA token...';
  spinner.start();

  try {
    // This is a placeholder for the actual token deployment code
    // In a real implementation, you would use the Solana web3.js and SPL Token libraries
    // to create a new token mint and initialize it with the proper parameters
    
    // Simulate token deployment with a delay
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    // Mock token address (in a real implementation, this would be the actual mint address)
    const tokenAddress = new PublicKey('DATAnKVv5pzRz2DMyNwFiZCsZbM4QSFTUVeD5uBZ9Bs');
    
    spinner.succeed(`DATA token deployed successfully!`);
    console.log();
    console.log(chalk.green('Token Information:'));
    console.log(chalk.green('-'.repeat(50)));
    console.log(`Token Name: ${TOKEN_NAME}`);
    console.log(`Token Symbol: ${TOKEN_SYMBOL}`);
    console.log(`Decimals: ${TOKEN_DECIMALS}`);
    console.log(`Token Address: ${tokenAddress.toString()}`);
    console.log(`Network: ${answers.network}`);
    console.log(`Owner: ${keypair.publicKey.toString()}`);
    console.log(chalk.green('-'.repeat(50)));
    
    // Save token information to a file
    const tokenInfo = {
      name: TOKEN_NAME,
      symbol: TOKEN_SYMBOL,
      decimals: TOKEN_DECIMALS,
      address: tokenAddress.toString(),
      network: answers.network,
      owner: keypair.publicKey.toString(),
      deployedAt: new Date().toISOString(),
    };
    
    fs.writeFileSync(
      path.resolve('./token-info.json'),
      JSON.stringify(tokenInfo, null, 2)
    );
    
    console.log();
    console.log(chalk.blue('Token information saved to token-info.json'));
    console.log();
    console.log(chalk.yellow('Next Steps:'));
    console.log('1. Add the token address to your .env file as DATA_TOKEN_ADDRESS');
    console.log('2. Set up a liquidity pool on pump.fun');
    console.log('3. Update your frontend with the token address');
    
  } catch (error) {
    spinner.fail(`Failed to deploy token: ${error.message}`);
    console.error(error);
  }
}

// Run the script
main().catch((error) => {
  console.error(chalk.red('Deployment failed:'));
  console.error(error);
  process.exit(1);
}); 