// console.log('Initializing worker')

function postError(error) {
  self.postMessage({ 
    type: 'error',
    error: error.message 
  })
}

// Global error handler
self.onerror = function(error) {
    console.error('Worker error:', error);
    self.postMessage({ 
        type: 'error',
        error: 'Worker encountered an unhandled error: ' + error.message,
    });
    self.close(); // Terminate worker after unhandled error
    return true; // Prevent error from propagating
};

// Unhandled promise rejection handler
self.onunhandledrejection = function(event) {
    console.error('Unhandled promise rejection:', event.reason);
    self.postMessage({ 
        type: 'error',
        error: 'Unhandled promise rejection in worker: ' + event.reason.message,
    });
    self.close(); // Terminate worker after unhandled rejection
};
    
self.onmessage = async event => {
    try {
        console.log('worker received event', event);
        const timeout = 60000;
        const jsonData = JSON.parse(event.data);
    
        const fetchWithRetry = async (retryCount) => {
          try {
            const controller = new AbortController();
            const timeoutId = setTimeout(() => controller.abort(), timeout);
            
            const response = await fetch(jsonData.url, {
              headers: jsonData.headers,
              signal: controller.signal,
              init: {
                credentials: 'same-origin'
              },
            }).catch(error => {
              if (error.name === 'AbortError') {
                postError(new Error('Request timeout after ' + timeout + 'ms'));
              } else if (error.name === 'TypeError') {
                // Network error
                throw new Error('Network error: ' + error.message);
              } else {
                postError(error);
              }
            });
            
            clearTimeout(timeoutId);
            
            if (!response.ok) {
              let errorText = await response.text().catch(() => 'No error details available');
              try {
                let j = JSON.parse(errorText);
                errorText = `${j.error}`
              } catch(_) { }
              postError(new Error(`HTTP error! status: ${response.status}, details: ${errorText}`));
            } else {
              const data = await response.arrayBuffer();
              self.postMessage({
                type: 'data',
                body: data
              });
            }
          } catch (error) {
            console.log('Fetch error:', error);
            if (retryCount <= 0) {
              self.postMessage({ 
                type: 'error',
                error: error.message
              });
              self.close(); // Terminate worker after all retries failed
            } else {
              console.log(`Retrying... (${retryCount} attempts remaining)`);
              fetchWithRetry(retryCount - 1);
            }
          }
        };
        fetchWithRetry(3);
    } catch (error) {
        console.error('Unhandled error in onmessage:', error);
        self.postMessage({ 
            type: 'error',
            error: error.message,
        });
        self.close(); // Terminate worker after unhandled error in message handler
    }
};

self.postMessage({ type: 'ready' });