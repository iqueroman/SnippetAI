<script>
  import { onMount, onDestroy } from "svelte";

  let selectedModel = "Please insert API key to fetch models";
  let chatMessages = [];
  let chatContainer;
  let showSettings = false;
  let showDropdown = false;
  let apiKey = "";
  let systemPrompt = "";
  let models = [];
  let filteredModels = [];
  let searchQuery = "";
  let isLoadingModels = false;
  let isMonitoringClipboard = false;
  let lastImageHash = "";
  let debugLog = [];

  onMount(() => {
    // Load saved settings from localStorage
    loadSettings();
    // Start event-driven clipboard monitoring
    startEventDrivenClipboardMonitoring();
  });

  onDestroy(() => {
    // Stop clipboard monitoring
    stopEventDrivenClipboardMonitoring();
  });

  function loadSettings() {
    const savedApiKey = localStorage.getItem('openrouter_api_key');
    const savedSystemPrompt = localStorage.getItem('system_prompt');
    
    if (savedApiKey) {
      apiKey = savedApiKey;
      fetchModels(); // This will also restore the saved model
    } else {
      searchQuery = selectedModel;
    }
    
    if (savedSystemPrompt) {
      systemPrompt = savedSystemPrompt;
    }
  }

  function saveSettings() {
    localStorage.setItem('openrouter_api_key', apiKey);
    localStorage.setItem('system_prompt', systemPrompt);
    if (selectedModel !== "Please insert API key to fetch models" && selectedModel !== "Search and select model" && selectedModel !== "Loading models...") {
      localStorage.setItem('selected_model', selectedModel);
    }
  }

  async function fetchModels() {
    if (!apiKey.trim()) {
      selectedModel = "Please insert API key to fetch models";
      models = [];
      return;
    }

    isLoadingModels = true;
    selectedModel = "Loading models...";
    
    try {
      const response = await fetch('https://openrouter.ai/api/v1/models', {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${apiKey.trim()}`,
          'Content-Type': 'application/json'
        }
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();
      
      if (data.data && Array.isArray(data.data)) {
        // Get ALL models and sort them alphabetically for easier searching
        models = data.data
          .sort((a, b) => {
            // Sort by name/id alphabetically
            const aName = (a.name || a.id).toLowerCase();
            const bName = (b.name || b.id).toLowerCase();
            return aName.localeCompare(bName);
          });

        filteredModels = models;
        
        // Check if we have a saved model to restore
        const savedModel = localStorage.getItem('selected_model');
        if (savedModel && models.some(m => (m.name || m.id) === savedModel)) {
          selectedModel = savedModel;
          searchQuery = savedModel;
        } else {
          selectedModel = models.length > 0 ? "Search and select model" : "No models available";
          searchQuery = "";
        }
      } else {
        throw new Error('Invalid API response format');
      }
    } catch (error) {
      console.error('Error fetching models:', error);
      selectedModel = "Error loading models - check API key";
      models = [];
    } finally {
      isLoadingModels = false;
    }
  }

  function handleModelSelect(model) {
    selectedModel = model.name || model.id || model;
    searchQuery = selectedModel; // Update search to show selected model
    showDropdown = false;
    saveSettings(); // Save the selected model
  }

  function filterModels() {
    if (!searchQuery.trim()) {
      filteredModels = models;
    } else {
      const query = searchQuery.toLowerCase();
      filteredModels = models.filter(model => {
        const modelName = (model.name || model.id).toLowerCase();
        const modelId = model.id.toLowerCase();
        return modelName.includes(query) || modelId.includes(query);
      });
    }
  }

  function handleSearchInput() {
    filterModels();
    if (!showDropdown && searchQuery.trim()) {
      showDropdown = true;
    }
  }

  function handleApiKeyChange() {
    saveSettings();
    if (apiKey.trim()) {
      fetchModels();
    } else {
      selectedModel = "Please insert API key to fetch models";
      models = [];
      filteredModels = [];
      searchQuery = "";
    }
  }

  function toggleDropdown() {
    if (models.length > 0 && !isLoadingModels) {
      showDropdown = !showDropdown;
      if (showDropdown) {
        // Focus search input when dropdown opens
        setTimeout(() => {
          const searchInput = document.getElementById('model-search');
          if (searchInput) searchInput.focus();
        }, 50);
      }
    }
  }

  function handleInputFocus() {
    // Only open dropdown if it's not already open
    if (models.length > 0 && !isLoadingModels && !showDropdown) {
      showDropdown = true;
    }
  }

  function handleArrowClick(event) {
    // Prevent the event from bubbling and causing input blur
    event.preventDefault();
    event.stopPropagation();
    
    if (models.length > 0 && !isLoadingModels) {
      showDropdown = !showDropdown;
      if (showDropdown) {
        // Focus search input when dropdown opens via arrow click
        setTimeout(() => {
          const searchInput = document.getElementById('model-search');
          if (searchInput) searchInput.focus();
        }, 50);
      }
    }
  }

  function handleSearchKeydown(event) {
    if (event.key === 'Escape') {
      showDropdown = false;
      searchQuery = selectedModel;
    } else if (event.key === 'Enter') {
      if (filteredModels.length > 0) {
        handleModelSelect(filteredModels[0]);
      }
    }
  }

  async function generateImageHash(blob) {
    const arrayBuffer = await blob.arrayBuffer();
    const hashBuffer = await crypto.subtle.digest('SHA-256', arrayBuffer);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    const hashHex = hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
    return hashHex;
  }

  async function startEventDrivenClipboardMonitoring() {
    if (isMonitoringClipboard) return;
    
    try {
      // Import Tauri API components separately
      const { invoke } = await import('@tauri-apps/api/tauri');
      const { listen } = await import('@tauri-apps/api/event');
      
      debugLog = [...debugLog, 'Tauri APIs imported successfully'];
      
      // Set up event listener for clipboard images from Tauri backend
      const unlistenImage = await listen('clipboard-image', async (event) => {
        debugLog = [...debugLog, 'Clipboard image event received'];
        const base64Data = event.payload;
        debugLog = [...debugLog, `Image data size: ${base64Data.length}`];
        
        // Convert base64 to blob and process
        try {
          const binaryString = atob(base64Data);
          const bytes = new Uint8Array(binaryString.length);
          for (let i = 0; i < binaryString.length; i++) {
            bytes[i] = binaryString.charCodeAt(i);
          }
          const blob = new Blob([bytes], { type: 'image/png' });
          debugLog = [...debugLog, `Converted to blob, size: ${blob.size}`];
          
          // Generate hash and add to chat
          const imageHash = await generateImageHash(blob);
          if (imageHash !== lastImageHash) {
            debugLog = [...debugLog, 'New image detected, adding to chat'];
            lastImageHash = imageHash;
            addScreenshotMessage(blob);
          } else {
            debugLog = [...debugLog, 'Same image as before, ignoring'];
          }
        } catch (e) {
          debugLog = [...debugLog, `Error processing image: ${e.message}`];
        }
      });

      // Set up event listener for clipboard text from Tauri backend
      const unlistenText = await listen('clipboard-text', async (event) => {
        debugLog = [...debugLog, 'Clipboard text event received'];
        const textData = event.payload;
        debugLog = [...debugLog, `Text data length: ${textData.length}`];
        
        try {
          // Add text message to chat
          debugLog = [...debugLog, 'New text detected, adding to chat'];
          addTextMessage(textData);
        } catch (e) {
          debugLog = [...debugLog, `Error processing text: ${e.message}`];
        }
      });
      
      // Set up fallback event listener for other clipboard updates
      const unlistenGeneral = await listen('clipboard-updated', async () => {
        debugLog = [...debugLog, 'General clipboard updated event received'];
        // Try browser clipboard as fallback (may fail if not focused)
        await checkForNewScreenshot();
      });
      
      // Store unlisten functions for cleanup
      window.clipboardUnlisten = () => {
        unlistenImage();
        unlistenText();
        unlistenGeneral();
      };
      
      // Start clipboard monitoring on Tauri backend
      const result = await invoke('start_clipboard_monitoring');
      debugLog = [...debugLog, `Tauri result: ${result}`];
      
      isMonitoringClipboard = true;
      debugLog = [...debugLog, 'Event-driven clipboard monitoring started successfully'];
    } catch (error) {
      debugLog = [...debugLog, `Failed to start clipboard monitoring: ${error}`];
      isMonitoringClipboard = false;
    }
  }

  async function stopEventDrivenClipboardMonitoring() {
    if (!isMonitoringClipboard) return;
    
    const { invoke } = await import('@tauri-apps/api/tauri');
    
    // Stop clipboard monitoring on Tauri backend
    await invoke('stop_clipboard_monitoring');
    
    // Clean up event listener
    if (window.clipboardUnlisten) {
      window.clipboardUnlisten();
      delete window.clipboardUnlisten;
    }
    
    isMonitoringClipboard = false;
    console.log('Event-driven clipboard monitoring stopped');
  }

  async function checkForNewScreenshot() {
    debugLog = [...debugLog, 'Checking for new screenshot...'];
    try {
      const clipboardItems = await navigator.clipboard.read();
      debugLog = [...debugLog, `Clipboard items: ${clipboardItems.length}`];
      
      for (const item of clipboardItems) {
        debugLog = [...debugLog, `Item types: ${item.types.join(', ')}`];
        if (item.types.includes('image/png')) {
          debugLog = [...debugLog, 'Found PNG image in clipboard'];
          const blob = await item.getType('image/png');
          debugLog = [...debugLog, `Image blob size: ${blob.size}`];
          
          // Generate a hash of the image content to detect duplicates
          const imageHash = await generateImageHash(blob);
          debugLog = [...debugLog, `Image hash: ${imageHash.substring(0, 8)}...`];
          
          // Only process if this is a new image (different hash)
          if (imageHash !== lastImageHash) {
            debugLog = [...debugLog, 'New image detected, processing...'];
            lastImageHash = imageHash;
            addScreenshotMessage(blob);
          } else {
            debugLog = [...debugLog, 'Same image as before, ignoring'];
          }
        }
      }
    } catch (err) {
      debugLog = [...debugLog, `Error checking clipboard: ${err.message}`];
    }
  }

  function addScreenshotMessage(imageBlob) {
    const imageUrl = URL.createObjectURL(imageBlob);
    const userMessage = {
      id: Date.now(),
      type: 'user',
      content: imageUrl,
      timestamp: new Date(),
      isScreenshot: true,
      imageBlob: imageBlob // Store blob for API call
    };
    
    chatMessages = [...chatMessages, userMessage];
    scrollToBottom();
    
    // Send to OpenRouter API
    sendScreenshotToAPI(userMessage);
  }

  function addTextMessage(text) {
    const userMessage = {
      id: Date.now(),
      type: 'user',
      content: text,
      timestamp: new Date(),
      isText: true
    };
    
    chatMessages = [...chatMessages, userMessage];
    scrollToBottom();
    
    // Send to OpenRouter API
    sendTextToAPI(userMessage);
  }

  async function sendScreenshotToAPI(userMessage) {
    // Check if we have all required data
    if (!apiKey.trim()) {
      addErrorMessage("Please set your OpenRouter API key in settings.");
      return;
    }

    if (!selectedModel || selectedModel === "Please insert API key to fetch models" || selectedModel === "Search and select model") {
      addErrorMessage("Please select a model from the dropdown.");
      return;
    }

    // Find the selected model object
    const modelObj = models.find(m => (m.name || m.id) === selectedModel);
    if (!modelObj) {
      addErrorMessage("Selected model not found. Please reselect a model.");
      return;
    }

    // Add loading message
    const loadingMessage = {
      id: Date.now() + 1,
      type: 'assistant',
      content: '',
      timestamp: new Date(),
      isLoading: true
    };
    chatMessages = [...chatMessages, loadingMessage];
    scrollToBottom();

    try {
      // Convert image blob to base64
      const base64Image = await blobToBase64(userMessage.imageBlob);
      
      // Prepare the API request
      const requestBody = {
        model: modelObj.id,
        messages: [
          ...(systemPrompt.trim() ? [{
            role: "system",
            content: systemPrompt.trim()
          }] : []),
          {
            role: "user",
            content: [
              {
                type: "image_url",
                image_url: {
                  url: base64Image
                }
              }
            ]
          }
        ],
        max_tokens: 1000,
        temperature: 0.7
      };

      const response = await fetch('https://openrouter.ai/api/v1/chat/completions', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${apiKey.trim()}`,
          'Content-Type': 'application/json',
          'HTTP-Referer': 'https://snippetai.app',
          'X-Title': 'SnippetAI'
        },
        body: JSON.stringify(requestBody)
      });

      if (!response.ok) {
        throw new Error(`API request failed: ${response.status} ${response.statusText}`);
      }

      const data = await response.json();
      
      if (data.choices && data.choices[0] && data.choices[0].message) {
        // Replace loading message with actual response
        chatMessages = chatMessages.map(msg => 
          msg.id === loadingMessage.id 
            ? {
                ...msg,
                content: data.choices[0].message.content,
                isLoading: false
              }
            : msg
        );
      } else {
        throw new Error('Invalid response format from API');
      }

    } catch (error) {
      console.error('Error calling OpenRouter API:', error);
      
      // Replace loading message with error
      chatMessages = chatMessages.map(msg => 
        msg.id === loadingMessage.id 
          ? {
              ...msg,
              content: `Error: ${error.message}. Please check your API key and selected model.`,
              isLoading: false,
              isError: true
            }
          : msg
      );
    }
    
    scrollToBottom();
  }

  async function sendTextToAPI(userMessage) {
    // Check if we have all required data
    if (!apiKey.trim()) {
      addErrorMessage("Please set your OpenRouter API key in settings.");
      return;
    }

    if (!selectedModel || selectedModel === "Please insert API key to fetch models" || selectedModel === "Search and select model") {
      addErrorMessage("Please select a model from the dropdown.");
      return;
    }

    // Find the selected model object
    const modelObj = models.find(m => (m.name || m.id) === selectedModel);
    if (!modelObj) {
      addErrorMessage("Selected model not found. Please reselect a model.");
      return;
    }

    // Add loading message
    const loadingMessage = {
      id: Date.now() + 1,
      type: 'assistant',
      content: '',
      timestamp: new Date(),
      isLoading: true
    };
    chatMessages = [...chatMessages, loadingMessage];
    scrollToBottom();

    try {
      // Prepare the API request for text
      const requestBody = {
        model: modelObj.id,
        messages: [
          ...(systemPrompt.trim() ? [{
            role: "system",
            content: systemPrompt.trim()
          }] : []),
          {
            role: "user",
            content: userMessage.content
          }
        ],
        max_tokens: 1000,
        temperature: 0.7
      };

      const response = await fetch('https://openrouter.ai/api/v1/chat/completions', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${apiKey.trim()}`,
          'Content-Type': 'application/json',
          'HTTP-Referer': 'https://snippetai.app',
          'X-Title': 'SnippetAI'
        },
        body: JSON.stringify(requestBody)
      });

      if (!response.ok) {
        throw new Error(`API request failed: ${response.status} ${response.statusText}`);
      }

      const data = await response.json();
      
      if (data.choices && data.choices[0] && data.choices[0].message) {
        // Replace loading message with actual response
        chatMessages = chatMessages.map(msg => 
          msg.id === loadingMessage.id 
            ? {
                ...msg,
                content: data.choices[0].message.content,
                isLoading: false
              }
            : msg
        );
      } else {
        throw new Error('Invalid response format from API');
      }

    } catch (error) {
      console.error('Error calling OpenRouter API:', error);
      
      // Replace loading message with error
      chatMessages = chatMessages.map(msg => 
        msg.id === loadingMessage.id 
          ? {
              ...msg,
              content: `Error: ${error.message}. Please check your API key and selected model.`,
              isLoading: false,
              isError: true
            }
          : msg
      );
    }
    
    scrollToBottom();
  }

  async function blobToBase64(blob) {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onloadend = () => resolve(reader.result);
      reader.onerror = reject;
      reader.readAsDataURL(blob);
    });
  }

  function addErrorMessage(message) {
    chatMessages = [...chatMessages, {
      id: Date.now(),
      type: 'assistant',
      content: message,
      timestamp: new Date(),
      isError: true
    }];
    scrollToBottom();
  }

  function scrollToBottom() {
    setTimeout(() => {
      if (chatContainer) {
        chatContainer.scrollTop = chatContainer.scrollHeight;
      }
    }, 100);
  }

  function expandImage(imageUrl) {
    // Create a modal overlay to show the full-size image
    const modal = document.createElement('div');
    modal.style.cssText = `
      position: fixed;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      background: rgba(0, 0, 0, 0.8);
      display: flex;
      justify-content: center;
      align-items: center;
      z-index: 10000;
      cursor: pointer;
      backdrop-filter: blur(5px);
    `;
    
    const img = document.createElement('img');
    img.src = imageUrl;
    img.style.cssText = `
      max-width: 90%;
      max-height: 90%;
      object-fit: contain;
      border-radius: 12px;
      box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
    `;
    
    modal.appendChild(img);
    document.body.appendChild(modal);
    
    // Close modal when clicked
    modal.addEventListener('click', () => {
      document.body.removeChild(modal);
    });
    
    // Close modal with Escape key
    const handleEscape = (e) => {
      if (e.key === 'Escape') {
        document.body.removeChild(modal);
        document.removeEventListener('keydown', handleEscape);
      }
    };
    document.addEventListener('keydown', handleEscape);
  }

  function toggleSettings() {
    showSettings = !showSettings;
    if (showSettings) {
      showDropdown = false; // Close dropdown when opening settings
    }
  }

  function closeSettings() {
    saveSettings();
    showSettings = false;
  }

  function handleClickOutside(event) {
    // Only close if clicking outside the entire model selector component
    if (showDropdown && !event.target.closest('.model-selector')) {
      showDropdown = false;
      // Clear focus from input when closing via outside click
      const searchInput = document.getElementById('model-search');
      if (searchInput) searchInput.blur();
    }
  }

  onMount(() => {
    document.addEventListener('click', handleClickOutside);
    return () => {
      document.removeEventListener('click', handleClickOutside);
    };
  });
</script>

<main class="glass-bg">
  <div class="app-container">
    {#if !showSettings}
      <!-- Main Chat Interface -->
      <!-- Top Row -->
      <div class="top-row">
        <div class="model-selector glass">
          <div class="search-container">
            <input
              id="model-search"
              type="text"
              class="model-search-input"
              bind:value={searchQuery}
              on:input={handleSearchInput}
              on:focus={handleInputFocus}
              on:keydown={handleSearchKeydown}
              placeholder={selectedModel}
              disabled={models.length === 0 || isLoadingModels}
            />
            <button 
              class="dropdown-toggle-btn"
              on:click={handleArrowClick}
              disabled={models.length === 0 || isLoadingModels}>
              <svg class="dropdown-icon" class:rotated={showDropdown} viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd" />
              </svg>
            </button>
          </div>
          
          {#if showDropdown && filteredModels.length > 0}
            <div class="dropdown-menu glass">
              <div class="model-count">
                {filteredModels.length} model{filteredModels.length !== 1 ? 's' : ''} found
              </div>
              <div class="model-list">
                {#each filteredModels as model}
                  <button class="dropdown-item" on:click={() => handleModelSelect(model)}>
                    <div class="model-info">
                      <span class="model-name">{model.name || model.id}</span>
                      {#if model.context_length}
                        <span class="model-context">{(model.context_length / 1000).toFixed(0)}K context</span>
                      {/if}
                    </div>
                  </button>
                {/each}
              </div>
            </div>
          {/if}
        </div>
        
        <button class="settings-btn glass" on:click={toggleSettings}>
          <svg class="settings-icon" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19.14,12.94c0.04-0.3,0.06-0.61,0.06-0.94c0-0.32-0.02-0.64-0.07-0.94l2.03-1.58c0.18-0.14,0.23-0.41,0.12-0.61 l-1.92-3.32c-0.12-0.22-0.37-0.29-0.59-0.22l-2.39,0.96c-0.5-0.38-1.03-0.7-1.62-0.94L14.4,2.81c-0.04-0.24-0.24-0.41-0.48-0.41 h-3.84c-0.24,0-0.43,0.17-0.47,0.41L9.25,5.35C8.66,5.59,8.12,5.92,7.63,6.29L5.24,5.33c-0.22-0.08-0.47,0-0.59,0.22L2.74,8.87 C2.62,9.08,2.66,9.34,2.86,9.48l2.03,1.58C4.84,11.36,4.82,11.69,4.82,12s0.02,0.64,0.07,0.94l-2.03,1.58 c-0.18,0.14-0.23,0.41-0.12,0.61l1.92,3.32c0.12,0.22,0.37,0.29,0.59,0.22l2.39-0.96c0.5,0.38,1.03,0.7,1.62,0.94l0.36,2.54 c0.05,0.24,0.24,0.41,0.48,0.41h3.84c0.24,0,0.44-0.17,0.47-0.41l0.36-2.54c0.59-0.24,1.13-0.56,1.62-0.94l2.39,0.96 c0.22,0.08,0.47,0,0.59-0.22l1.92-3.32c0.12-0.22,0.07-0.47-0.12-0.61L19.14,12.94z M12,15.6c-1.98,0-3.6-1.62-3.6-3.6 s1.62-3.6,3.6-3.6s3.6,1.62,3.6,3.6S13.98,15.6,12,15.6z"/>
          </svg>
        </button>
      </div>

      <!-- Chat Area -->
      <div class="chat-area glass">
        <div class="chat-container" bind:this={chatContainer}>
          {#if chatMessages.length === 0}
            <div class="welcome-message">
              <div class="welcome-icon">🖼️</div>
              <h2>Welcome to SnippetAI</h2>
              <p>Press <kbd>Win + Shift + S</kbd> to select a region and start using the tool</p>
              
              <!-- Debug Log -->
              {#if debugLog.length > 0}
                <div class="debug-log">
                  <h3>Debug Log:</h3>
                  {#each debugLog.slice(-10) as log}
                    <div class="debug-entry">{log}</div>
                  {/each}
                </div>
              {/if}
            </div>
          {:else}
            {#each chatMessages as message (message.id)}
              <div class="message {message.type}">
                <div class="message-bubble glass" class:loading={message.isLoading} class:error={message.isError}>
                  {#if message.type === 'user'}
                    {#if message.isScreenshot}
                      <div 
                        class="image-message" 
                        class:screenshot={message.isScreenshot} 
                        role="button"
                        tabindex="0"
                        on:click={() => expandImage(message.content)}
                        on:keydown={(e) => {
                          if (e.key === 'Enter' || e.key === ' ') {
                            e.preventDefault();
                            expandImage(message.content);
                          }
                        }}
                        aria-label="Click to expand image"
                      >
                        <img src={message.content} alt="User screenshot" />
                      </div>
                    {:else if message.isText}
                      <div class="text-message user-text">
                        {message.content}
                      </div>
                    {/if}
                  {:else}
                    <div class="text-message">
                      {#if message.isLoading}
                        <div class="typing-animation">
                          <span class="typing-dot"></span>
                          <span class="typing-dot"></span>
                          <span class="typing-dot"></span>
                        </div>
                      {:else}
                        {message.content}
                      {/if}
                    </div>
                  {/if}
                </div>
              </div>
            {/each}
          {/if}
        </div>
      </div>
    {:else}
      <!-- Settings Screen -->
      <div class="settings-screen">
        <!-- Settings Header with Back Button -->
        <div class="settings-header">
          <button class="back-btn glass" on:click={closeSettings}>
            <svg class="back-icon" viewBox="0 0 24 24" fill="currentColor">
              <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.42-1.41L7.83 13H20v-2z"/>
            </svg>
            Back
          </button>
          <h2 class="settings-title">Settings</h2>
        </div>

        <!-- API Key Field -->
        <div class="settings-field">
          <label for="api-key" class="field-label">Open Router API Key</label>
          <input 
            type="password" 
            id="api-key" 
            class="api-key-input glass" 
            bind:value={apiKey}
            on:input={handleApiKeyChange}
            placeholder="Enter your OpenRouter API key"
          />
        </div>

        <!-- System Prompt Field -->
        <div class="settings-field system-prompt-field">
          <label for="system-prompt" class="field-label">System Prompt</label>
          <textarea 
            id="system-prompt" 
            class="system-prompt-input glass" 
            bind:value={systemPrompt}
            on:input={saveSettings}
            placeholder="Enter your system prompt here..."
          ></textarea>
        </div>
      </div>
    {/if}
  </div>
</main>

<style>
  :global(*) {
    box-sizing: border-box;
  }

  :global(html) {
    margin: 0;
    padding: 0;
    background: #ffffff;
    min-height: 100vh;
  }

  :global(body) {
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Helvetica Neue', Arial, sans-serif;
    background: #ffffff;
    min-height: 100vh;
    overflow: hidden;
  }

  main.glass-bg {
    width: 100vw;
    height: 100vh;
    padding: 0;
    display: flex;
    flex-direction: column;
    background: #ffffff;
    backdrop-filter: none;
    border: none;
    border-radius: 0;
    position: fixed;
    top: 0;
    left: 0;
  }

  .app-container {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding: 20px;
    min-width: 320px;
    box-sizing: border-box;
  }

  .top-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 20px;
    flex-shrink: 0;
  }

  .glass {
    background: rgba(255, 255, 255, 0.15);
    backdrop-filter: blur(25px);
    border-radius: 16px;
    border: 1px solid rgba(255, 255, 255, 0.25);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
  }

  .model-selector {
    flex: 1;
    position: relative;
    z-index: 10;
    min-width: 0;
  }

  .search-container {
    display: flex;
    align-items: center;
    width: 100%;
  }

  .model-search-input {
    flex: 1;
    padding: 14px 18px;
    background: transparent;
    border: none;
    color: #333333;
    font-size: 16px;
    font-weight: 500;
    outline: none;
    border-radius: 16px 0 0 16px;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .model-search-input:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .model-search-input::placeholder {
    color: #666666;
    opacity: 0.8;
  }

  .dropdown-toggle-btn {
    padding: 14px 12px;
    background: transparent;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 0 16px 16px 0;
    transition: all 0.2s ease;
    border-left: 1px solid rgba(255, 255, 255, 0.2);
    outline: none;
    user-select: none;
  }

  .dropdown-toggle-btn:hover {
    background: rgba(0, 0, 0, 0.05);
  }

  .dropdown-toggle-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .dropdown-toggle-btn:disabled:hover {
    background: transparent;
  }

  .dropdown-icon {
    width: 20px;
    height: 20px;
    color: #666666;
    transition: transform 0.2s ease;
  }

  .dropdown-icon.rotated {
    transform: rotate(180deg);
  }

  .dropdown-menu {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    margin-top: 8px;
    z-index: 1000;
    max-height: 400px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    background: rgba(255, 255, 255, 0.95);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(0, 0, 0, 0.1);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  }

  .model-count {
    padding: 12px 16px;
    background: rgba(0, 0, 0, 0.03);
    font-size: 14px;
    color: #666666;
    font-weight: 500;
    border-bottom: 1px solid rgba(0, 0, 0, 0.1);
  }

  .model-list {
    overflow-y: auto;
    max-height: 350px;
  }


  .dropdown-item {
    width: 100%;
    padding: 12px 16px;
    background: transparent;
    border: none;
    color: #333333;
    font-size: 15px;
    cursor: pointer;
    text-align: left;
    transition: all 0.2s ease;
    border-radius: 0;
  }

  .dropdown-item:first-child {
    border-radius: 16px 16px 0 0;
  }

  .dropdown-item:last-child {
    border-radius: 0 0 16px 16px;
  }

  .dropdown-item:only-child {
    border-radius: 16px;
  }

  .dropdown-item:hover {
    background: rgba(0, 0, 0, 0.08);
  }

  .model-info {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
  }

  .model-name {
    font-size: 15px;
    font-weight: 500;
    color: #333333;
  }

  .model-context {
    font-size: 12px;
    color: #666666;
    opacity: 0.8;
  }

  .settings-btn {
    width: 56px;
    height: 56px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    cursor: pointer;
    transition: all 0.2s ease;
    border-radius: 16px;
    z-index: 10;
  }

  .settings-btn:hover {
    background: rgba(0, 0, 0, 0.05);
  }

  .settings-icon {
    width: 24px;
    height: 24px;
    color: #666666;
  }

  .chat-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    position: relative;
    min-height: 0;
    cursor: pointer;
    z-index: 5;
    overflow: hidden;
  }


  .chat-container {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .welcome-message {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
    color: #333333;
    padding: 40px;
  }

  .welcome-icon {
    font-size: 48px;
    margin-bottom: 20px;
  }

  .welcome-message h2 {
    margin: 0 0 12px 0;
    font-size: 28px;
    font-weight: 600;
    color: #222222;
  }

  .welcome-message p {
    margin: 0 0 30px 0;
    font-size: 16px;
    color: #555555;
    max-width: 400px;
    line-height: 1.5;
  }

  .welcome-message kbd {
    background: rgba(0, 0, 0, 0.1);
    border: 1px solid rgba(0, 0, 0, 0.2);
    border-radius: 4px;
    padding: 2px 6px;
    font-size: 14px;
    font-family: monospace;
    font-weight: 600;
    color: #333333;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
  }

  .debug-log {
    margin-top: 20px;
    padding: 10px;
    background: rgba(0, 0, 0, 0.1);
    border-radius: 8px;
    max-width: 600px;
    text-align: left;
  }

  .debug-log h3 {
    margin: 0 0 10px 0;
    font-size: 14px;
    color: #333333;
  }

  .debug-entry {
    font-family: monospace;
    font-size: 12px;
    color: #555555;
    margin: 2px 0;
    padding: 2px 4px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
  }


  .message {
    display: flex;
    margin-bottom: 16px;
  }

  .message.user {
    justify-content: flex-end;
  }

  .message.assistant {
    justify-content: flex-start;
  }

  .message-bubble {
    max-width: min(70%, 400px);
    padding: 12px 16px;
    border-radius: 18px;
    position: relative;
    word-wrap: break-word;
    overflow-wrap: break-word;
  }

  .message.user .message-bubble {
    background: rgba(100, 149, 237, 0.2);
    border-bottom-right-radius: 6px;
  }

  .message.assistant .message-bubble {
    background: rgba(255, 255, 255, 0.15);
    border-bottom-left-radius: 6px;
  }

  .image-message {
    border-radius: 12px;
    overflow: hidden;
    max-width: min(300px, 80vw);
    cursor: pointer;
    transition: all 0.2s ease;
    outline: none;
  }

  .image-message:focus {
    outline: 2px solid #4CAF50;
    outline-offset: 2px;
  }

  .image-message.screenshot {
    max-width: min(200px, 60vw);
    max-height: 150px;
  }

  .image-message.screenshot:hover {
    transform: scale(1.02);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  .image-message img {
    width: 100%;
    height: auto;
    display: block;
    border-radius: 12px;
  }

  .image-message.screenshot img {
    object-fit: cover;
    height: 100%;
    max-height: 150px;
  }

  .text-message {
    color: #333333;
    font-size: 15px;
    line-height: 1.4;
    word-wrap: break-word;
  }

  .text-message.user-text {
    color: #2c5282;
    font-weight: 500;
  }

  /* Loading and error message styles */
  .message-bubble.loading {
    background: rgba(100, 149, 237, 0.1);
  }

  .typing-animation {
    display: flex;
    align-items: center;
    gap: 3px;
    padding: 4px 0;
  }

  .typing-dot {
    width: 6px;
    height: 6px;
    background-color: #666666;
    border-radius: 50%;
    opacity: 0.4;
    animation: typingDot 1.4s infinite ease-in-out both;
  }

  .typing-dot:nth-child(1) {
    animation-delay: -0.32s;
  }

  .typing-dot:nth-child(2) {
    animation-delay: -0.16s;
  }

  .typing-dot:nth-child(3) {
    animation-delay: 0s;
  }

  @keyframes typingDot {
    0%, 80%, 100% {
      opacity: 0.4;
      transform: scale(1);
    }
    40% {
      opacity: 1;
      transform: scale(1.2);
    }
  }

  .message-bubble.error {
    background: rgba(220, 53, 69, 0.1);
    border-left: 3px solid #dc3545;
  }

  .message-bubble.error .text-message {
    color: #dc3545;
  }


  /* Custom scrollbar */
  .chat-container::-webkit-scrollbar {
    width: 6px;
  }

  .chat-container::-webkit-scrollbar-track {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
  }

  .chat-container::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.3);
    border-radius: 3px;
  }

  .chat-container::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.4);
  }

  /* Smooth animations */
  .message-bubble {
    animation: slideIn 0.3s ease-out;
  }

  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateY(20px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .message.user .message-bubble {
    animation: slideInRight 0.3s ease-out;
  }

  @keyframes slideInRight {
    from {
      opacity: 0;
      transform: translateX(20px);
    }
    to {
      opacity: 1;
      transform: translateX(0);
    }
  }

  /* Settings Screen Styles */
  .settings-screen {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding: 19px;
  }

  .settings-header {
    display: flex;
    align-items: center;
    gap: 20px;
  }

  .back-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 18px;
    background: transparent;
    border: none;
    color: #333333;
    font-size: 16px;
    font-weight: 500;
    cursor: pointer;
    border-radius: 12px;
    transition: all 0.2s ease;
    z-index: 10;
  }

  .back-btn:hover {
    background: rgba(0, 0, 0, 0.05);
    transform: translateX(-2px);
  }

  .back-icon {
    width: 20px;
    height: 20px;
  }

  .settings-title {
    margin: 0;
    font-size: 24px;
    font-weight: 600;
    color: #222222;
    flex: 1;
  }

  .settings-field {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .system-prompt-field {
    flex: 1;
    min-height: 0;
  }

  .field-label {
    font-size: 16px;
    font-weight: 500;
    color: #333333;
    margin: 0;
  }

  .api-key-input {
    width: 100%;
    height: 56px;
    padding: 14px 18px;
    background: transparent;
    border: none;
    color: #333333;
    font-size: 16px;
    border-radius: 16px;
    outline: none;
    transition: all 0.2s ease;
    z-index: 10;
  }

  .api-key-input:focus {
    background: rgba(255, 255, 255, 0.2);
    border-color: rgba(255, 255, 255, 0.4);
  }

  .api-key-input::placeholder {
    color: #666666;
    opacity: 0.8;
  }

  .system-prompt-input {
    width: 100%;
    flex: 1;
    min-height: 200px;
    padding: 18px;
    background: transparent;
    border: none;
    color: #333333;
    font-size: 16px;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Helvetica Neue', Arial, sans-serif;
    border-radius: 16px;
    outline: none;
    resize: none;
    transition: all 0.2s ease;
    z-index: 10;
    line-height: 1.5;
  }

  .system-prompt-input:focus {
    background: rgba(255, 255, 255, 0.2);
    border-color: rgba(255, 255, 255, 0.4);
  }

  .system-prompt-input::placeholder {
    color: #666666;
    opacity: 0.8;
  }

  /* Custom scrollbar for textarea */
  .system-prompt-input::-webkit-scrollbar {
    width: 6px;
  }

  .system-prompt-input::-webkit-scrollbar-track {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
  }

  .system-prompt-input::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.3);
    border-radius: 3px;
  }

  .system-prompt-input::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.4);
  }

  /* Responsive Design */
  @media (max-width: 900px) {
    .app-container {
      padding: 16px;
      gap: 16px;
    }

    .top-row {
      flex-wrap: wrap;
      gap: 12px;
    }

    .model-selector {
      flex: 1;
      min-width: 300px;
    }

    .settings-btn {
      width: 48px;
      height: 48px;
    }
  }

  @media (max-width: 768px) {
    .app-container {
      padding: 14px;
      gap: 14px;
    }

    .top-row {
      flex-direction: column;
      gap: 12px;
      align-items: stretch;
    }

    .model-selector {
      width: 100%;
      min-width: 0;
    }

    .settings-btn {
      width: 48px;
      height: 48px;
      align-self: flex-end;
      position: absolute;
      top: 14px;
      right: 14px;
      z-index: 20;
    }

    .message-bubble {
      max-width: 85%;
      padding: 10px 14px;
    }

    .welcome-message h2 {
      font-size: 24px;
    }

    .welcome-message p {
      font-size: 14px;
    }

    .chat-container {
      padding: 16px;
    }

    .model-search-input {
      font-size: 14px;
      padding: 12px 16px;
    }

    .dropdown-toggle-btn {
      padding: 12px 10px;
    }
  }

  @media (max-width: 600px) {
    .top-row {
      padding-top: 48px; /* Make room for absolutely positioned settings button */
    }
  }

  @media (max-width: 480px) {
    .app-container {
      padding: 10px;
      gap: 10px;
    }

    .top-row {
      padding-top: 50px;
    }

    .settings-btn {
      top: 10px;
      right: 10px;
      width: 40px;
      height: 40px;
    }

    .message-bubble {
      max-width: 90%;
      padding: 8px 12px;
      font-size: 14px;
    }

    .chat-container {
      padding: 10px;
    }

    .welcome-message {
      padding: 15px;
    }

    .welcome-message h2 {
      font-size: 20px;
    }

    .welcome-message p {
      font-size: 13px;
    }

    .image-message {
      max-width: 90vw;
    }

    .image-message.screenshot {
      max-width: 75vw;
    }

    .model-search-input {
      font-size: 13px;
      padding: 10px 14px;
    }

    .dropdown-toggle-btn {
      padding: 10px 8px;
    }

    .settings-icon {
      width: 20px;
      height: 20px;
    }
  }

  /* Ensure dropdown doesn't overflow on small screens */
  .dropdown-menu {
    max-width: calc(100vw - 40px);
    left: 0;
    right: 0;
  }

  @media (max-width: 768px) {
    .dropdown-menu {
      max-width: calc(100vw - 32px);
    }

    /* Settings screen responsive adjustments */
    .settings-screen {
      padding: 14px;
      gap: 16px;
    }

    .back-btn {
      padding: 10px 16px;
      font-size: 14px;
    }

    .settings-title {
      font-size: 20px;
    }

    .api-key-input {
      height: 48px;
      padding: 12px 16px;
      font-size: 14px;
    }

    .system-prompt-input {
      padding: 16px;
      font-size: 14px;
      min-height: 150px;
    }
  }

  @media (max-width: 480px) {
    .dropdown-menu {
      max-width: calc(100vw - 24px);
    }

    /* Settings screen small screen adjustments */
    .settings-screen {
      padding: 10px;
      gap: 12px;
    }

    .settings-header {
      gap: 12px;
    }

    .back-btn {
      padding: 8px 12px;
      font-size: 13px;
    }

    .settings-title {
      font-size: 18px;
    }

    .field-label {
      font-size: 14px;
    }

    .api-key-input {
      height: 44px;
      padding: 10px 14px;
      font-size: 13px;
    }

    .system-prompt-input {
      padding: 14px;
      font-size: 13px;
      min-height: 120px;
    }

    .back-icon {
      width: 18px;
      height: 18px;
    }
  }
</style>