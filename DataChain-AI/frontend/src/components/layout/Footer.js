import React from 'react';
import { Link } from 'react-router-dom';
import { FaTwitter, FaDiscord, FaGithub, FaTelegram } from 'react-icons/fa';

const Footer = () => {
  return (
    <footer className="bg-gray-800 text-white py-8">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-8">
          <div>
            <h3 className="text-lg font-semibold mb-4">DataChain AI</h3>
            <p className="text-gray-300 text-sm">
              A decentralized AI-powered data analytics platform built on the Solana blockchain.
            </p>
          </div>
          <div>
            <h3 className="text-lg font-semibold mb-4">Navigation</h3>
            <ul className="space-y-2 text-gray-300 text-sm">
              <li>
                <Link to="/dashboard" className="hover:text-indigo-400">Dashboard</Link>
              </li>
              <li>
                <Link to="/marketplace" className="hover:text-indigo-400">Data Marketplace</Link>
              </li>
              <li>
                <Link to="/ai-models" className="hover:text-indigo-400">AI Models</Link>
              </li>
              <li>
                <Link to="/governance" className="hover:text-indigo-400">Governance</Link>
              </li>
            </ul>
          </div>
          <div>
            <h3 className="text-lg font-semibold mb-4">Resources</h3>
            <ul className="space-y-2 text-gray-300 text-sm">
              <li>
                <a href="https://docs.datachain.ai" target="_blank" rel="noopener noreferrer" className="hover:text-indigo-400">
                  Documentation
                </a>
              </li>
              <li>
                <a href="https://github.com/datachainai" target="_blank" rel="noopener noreferrer" className="hover:text-indigo-400">
                  GitHub
                </a>
              </li>
              <li>
                <a href="https://datachain.ai/whitepaper" target="_blank" rel="noopener noreferrer" className="hover:text-indigo-400">
                  Whitepaper
                </a>
              </li>
              <li>
                <a href="https://datachain.ai/faq" target="_blank" rel="noopener noreferrer" className="hover:text-indigo-400">
                  FAQ
                </a>
              </li>
            </ul>
          </div>
          <div>
            <h3 className="text-lg font-semibold mb-4">Community</h3>
            <div className="flex space-x-4">
              <a href="https://twitter.com/DataChain_AI" target="_blank" rel="noopener noreferrer" className="text-gray-300 hover:text-indigo-400">
                <FaTwitter size={24} />
              </a>
              <a href="https://discord.gg/datachainai" target="_blank" rel="noopener noreferrer" className="text-gray-300 hover:text-indigo-400">
                <FaDiscord size={24} />
              </a>
              <a href="https://github.com/datachainai" target="_blank" rel="noopener noreferrer" className="text-gray-300 hover:text-indigo-400">
                <FaGithub size={24} />
              </a>
              <a href="https://t.me/datachainai" target="_blank" rel="noopener noreferrer" className="text-gray-300 hover:text-indigo-400">
                <FaTelegram size={24} />
              </a>
            </div>
          </div>
        </div>
        <div className="mt-8 pt-8 border-t border-gray-700 flex flex-col md:flex-row justify-between items-center">
          <p className="text-gray-400 text-sm">© {new Date().getFullYear()} DataChain AI. All rights reserved.</p>
          <div className="mt-4 md:mt-0 flex space-x-6">
            <Link to="/privacy" className="text-gray-400 hover:text-indigo-400 text-sm">
              Privacy Policy
            </Link>
            <Link to="/terms" className="text-gray-400 hover:text-indigo-400 text-sm">
              Terms of Service
            </Link>
          </div>
        </div>
      </div>
    </footer>
  );
};

export default Footer; 