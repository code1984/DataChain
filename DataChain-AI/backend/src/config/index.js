require('dotenv').config();

const config = {
  // Server configuration
  env: process.env.NODE_ENV || 'development',
  port: process.env.PORT || 4000,
  
  // Database settings
  database: {
    url: process.env.DATABASE_URL,
    options: {
      useNewUrlParser: true,
      useUnifiedTopology: true,
    }
  },
  
  // Redis configuration
  redis: {
    url: process.env.REDIS_URL,
    options: {}
  },
  
  // CORS settings
  cors: {
    origin: process.env.CORS_ORIGIN ? 
      process.env.CORS_ORIGIN.split(',') : 
      ['http://localhost:3000', 'https://datachain.ai'],
    credentials: true
  },
  
  // Authentication
  jwt: {
    secret: process.env.JWT_SECRET || 'your_jwt_secret_key_for_development',
    expiresIn: process.env.JWT_EXPIRATION || '24h'
  },
  
  // Solana blockchain
  solana: {
    network: process.env.SOLANA_NETWORK || 'devnet',
    rpcUrl: process.env.SOLANA_RPC_URL || 'https://api.devnet.solana.com',
    walletPath: process.env.SOLANA_WALLET_PATH || './keypair.json',
    dataTokenAddress: process.env.DATA_TOKEN_ADDRESS
  },
  
  // AI Engine
  aiEngine: {
    url: process.env.AI_ENGINE_URL || 'http://localhost:5000',
    apiKey: process.env.AI_ENGINE_API_KEY
  },
  
  // External APIs
  openai: {
    apiKey: process.env.OPENAI_API_KEY
  },
  
  huggingface: {
    apiKey: process.env.HUGGINGFACE_API_KEY
  },
  
  // Logging
  logLevel: process.env.LOG_LEVEL || 'info',
  
  // Email settings
  email: {
    host: process.env.SMTP_HOST,
    port: parseInt(process.env.SMTP_PORT || '587', 10),
    user: process.env.SMTP_USER,
    pass: process.env.SMTP_PASS,
    from: process.env.EMAIL_FROM || 'noreply@datachain.ai'
  },
  
  // Storage
  storage: {
    type: process.env.STORAGE_TYPE || 'local', // 'local', 's3', 'gcp'
    path: process.env.STORAGE_PATH || './uploads',
    s3: {
      bucket: process.env.AWS_S3_BUCKET,
      accessKeyId: process.env.AWS_ACCESS_KEY_ID,
      secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY,
      region: process.env.AWS_REGION || 'us-east-1'
    }
  },
  
  // Feature flags
  features: {
    dataMarketplace: process.env.ENABLE_DATA_MARKETPLACE !== 'false',
    aiModelMarketplace: process.env.ENABLE_AI_MODEL_MARKETPLACE !== 'false',
    governance: process.env.ENABLE_GOVERNANCE !== 'false',
    predictions: process.env.ENABLE_PREDICTIONS !== 'false'
  }
};

// Environment-specific configuration
if (config.env === 'production') {
  // Additional production-specific settings
  config.cors.origin = ['https://datachain.ai', 'https://app.datachain.ai'];
} else if (config.env === 'test') {
  // Test-specific settings
  config.database.url = process.env.TEST_DATABASE_URL || config.database.url;
}

module.exports = config; 