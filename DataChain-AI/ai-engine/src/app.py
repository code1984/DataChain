import os
import logging
from flask import Flask, request, jsonify
from flask_cors import CORS
from dotenv import load_dotenv
import traceback

# Load environment variables
load_dotenv()

# Import modules
from .config import Config
from .models import model_manager
from .processors import data_processor
from .insights import insight_generator
from .utils import logger

# Initialize Flask app
app = Flask(__name__)
CORS(app)

# Configure logging
log_level = os.getenv('LOG_LEVEL', 'INFO').upper()
logger.setup_logger(log_level)
app.logger.setLevel(getattr(logging, log_level))

# Load configuration
config = Config()

# Initialize model manager
model_manager.init_models(config.MODEL_CACHE_DIR)

@app.route('/health', methods=['GET'])
def health_check():
    """Health check endpoint"""
    return jsonify({
        'status': 'ok',
        'version': config.VERSION,
        'models_loaded': model_manager.get_loaded_models()
    })

@app.route('/api/analyze', methods=['POST'])
def analyze_data():
    """Analyze data and return insights"""
    try:
        data = request.json
        
        if not data or 'dataset' not in data:
            return jsonify({'error': 'No dataset provided'}), 400
        
        # Process the data
        processed_data = data_processor.process_data(data['dataset'])
        
        # Generate insights
        insights = insight_generator.generate_insights(
            processed_data, 
            model_name=data.get('model', 'default'),
            params=data.get('params', {})
        )
        
        return jsonify({
            'insights': insights,
            'metadata': {
                'dataset_size': len(data['dataset']),
                'processing_time': processed_data.get('processing_time', 0),
                'model_used': insights.get('model_used', 'default')
            }
        })
    except Exception as e:
        app.logger.error(f"Error in analyze_data: {str(e)}")
        app.logger.error(traceback.format_exc())
        return jsonify({'error': str(e)}), 500

@app.route('/api/query', methods=['POST'])
def natural_language_query():
    """Process natural language query on data"""
    try:
        data = request.json
        
        if not data or 'query' not in data or 'dataset' not in data:
            return jsonify({'error': 'Query and dataset are required'}), 400
        
        # Process the query
        result = model_manager.process_query(
            query=data['query'],
            dataset=data['dataset'],
            model_name=data.get('model', 'default')
        )
        
        return jsonify({
            'result': result,
            'metadata': {
                'query': data['query'],
                'model_used': result.get('model_used', 'default')
            }
        })
    except Exception as e:
        app.logger.error(f"Error in natural_language_query: {str(e)}")
        app.logger.error(traceback.format_exc())
        return jsonify({'error': str(e)}), 500

@app.route('/api/predict', methods=['POST'])
def predict():
    """Make predictions based on data"""
    try:
        data = request.json
        
        if not data or 'dataset' not in data or 'target' not in data:
            return jsonify({'error': 'Dataset and target are required'}), 400
        
        # Process the prediction
        prediction = model_manager.make_prediction(
            dataset=data['dataset'],
            target=data['target'],
            features=data.get('features', []),
            model_name=data.get('model', 'default'),
            params=data.get('params', {})
        )
        
        return jsonify({
            'prediction': prediction,
            'metadata': {
                'model_used': prediction.get('model_used', 'default'),
                'accuracy': prediction.get('accuracy', 0)
            }
        })
    except Exception as e:
        app.logger.error(f"Error in predict: {str(e)}")
        app.logger.error(traceback.format_exc())
        return jsonify({'error': str(e)}), 500

@app.route('/api/models', methods=['GET'])
def list_models():
    """List available models"""
    try:
        models = model_manager.list_available_models()
        return jsonify({'models': models})
    except Exception as e:
        app.logger.error(f"Error in list_models: {str(e)}")
        app.logger.error(traceback.format_exc())
        return jsonify({'error': str(e)}), 500

@app.errorhandler(404)
def not_found(e):
    return jsonify({'error': 'Endpoint not found'}), 404

@app.errorhandler(405)
def method_not_allowed(e):
    return jsonify({'error': 'Method not allowed'}), 405

@app.errorhandler(500)
def server_error(e):
    app.logger.error(f"Server error: {str(e)}")
    return jsonify({'error': 'Internal server error'}), 500

if __name__ == '__main__':
    port = int(os.getenv('PORT', 5000))
    debug = os.getenv('FLASK_ENV', 'production') == 'development'
    app.run(host='0.0.0.0', port=port, debug=debug) 