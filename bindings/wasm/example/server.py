from flask import Flask, send_from_directory, request
from flask_cors import CORS
from requests import request as req
import os

current_dir = os.path.dirname(os.path.abspath(__file__))
parent_dir = os.path.dirname(current_dir)

app = Flask(__name__, static_folder=current_dir, static_url_path='')

CORS(app)

SITE_NAME = 'http://localhost:8000'

@app.route('/v1/<path:path>', methods=['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'OPTIONS'])
def proxy(path):
    headers = dict(request.headers)

    headers.pop('Host', None)
    headers.pop('Content-Length', None)
    
    response = req(
        method=request.method,
        url=f'{SITE_NAME}/v1/{path}',
        headers=headers,
        data=request.get_data(),
        params=request.args,
        allow_redirects=False
    )
    
    flask_response = app.make_response(response.content)
    flask_response.status_code = response.status_code
    
    for key, value in response.headers.items():
        if key.lower() not in ['content-encoding', 'content-length', 'transfer-encoding', 'connection']:
            flask_response.headers[key] = value
    
    return flask_response

@app.route('/')
def serve_index():
    return send_from_directory(current_dir, 'index.html')

@app.route('/pkg/<path:filename>')
def serve_pkg(filename):
    pkg_dir = os.path.join(parent_dir, 'pkg')
    print('pkg', pkg_dir)
    return send_from_directory(pkg_dir, filename)

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=8123, debug=True)





