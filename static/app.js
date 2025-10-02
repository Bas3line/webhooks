let deleteWebhookId = null;

async function apiCall(url, options = {}) {
    try {
        console.log('Making API call to:', url);
        const response = await fetch(url, {
            headers: {
                'Content-Type': 'application/json',
                ...options.headers
            },
            ...options
        });

        console.log('Response status:', response.status);

        if (!response.ok) {
            console.error(`HTTP error! status: ${response.status}`);
            throw new Error(`HTTP error! status: ${response.status}`);
        }

        // Check if response has content before trying to parse JSON
        const contentType = response.headers.get('content-type');
        if (contentType && contentType.includes('application/json')) {
            const text = await response.text();
            if (text.trim()) {
                const result = JSON.parse(text);
                console.log('API result:', result);
                return result;
            }
        }

        // Return empty object for responses without JSON content
        return {};
    } catch (error) {
        console.error('API call failed:', error);
        throw error;
    }
}

async function loadWebhooks() {
    try {
        console.log('Loading webhooks...');
        const webhooks = await apiCall('/api/webhooks');
        console.log('Webhooks loaded:', webhooks);
        renderWebhooks(webhooks);
        updateStats(webhooks);
    } catch (error) {
        console.error('Failed to load webhooks:', error);
        document.getElementById('webhooksList').innerHTML = '<p>Failed to load webhooks: ' + error.message + '</p>';
    }
}

async function loadRecentEvents() {
    try {
        const events = await apiCall('/api/events?limit=5');
        renderRecentEvents(events);
    } catch (error) {
        console.error('Failed to load recent events:', error);
    }
}

function renderWebhooks(webhooks) {
    const container = document.getElementById('webhooksList');

    if (webhooks.length === 0) {
        container.innerHTML = '<p>No webhooks created yet. Create your first webhook above!</p>';
        return;
    }

    container.innerHTML = webhooks.map(webhook => `
        <div class="webhook-item">
            <div class="webhook-header">
                <div>
                    <div class="webhook-name">${webhook.name}</div>
                    <span class="status ${webhook.is_active ? 'status-active' : 'status-inactive'}">
                        ${webhook.is_active ? 'Active' : 'Inactive'}
                    </span>
                </div>
            </div>

            ${webhook.description ? `<p style="color: #6b7280; margin: 5px 0;">${webhook.description}</p>` : ''}

            <div class="webhook-endpoint">Endpoint: /${webhook.endpoint}</div>
            <div class="webhook-url">
                <strong>URL:</strong> ${window.location.origin}/webhook/${webhook.endpoint}
            </div>
            <div class="webhook-secret">
                <strong>Secret:</strong> ${webhook.secret}
            </div>

            <div style="margin-top: 10px; color: #6b7280; font-size: 12px;">
                Events: ${webhook.event_count || 0} | Created: ${new Date(webhook.created_at).toLocaleDateString()}
            </div>

            <div class="actions">
                <button class="btn btn-small" onclick="toggleWebhook('${webhook.id}', ${!webhook.is_active})">
                    ${webhook.is_active ? 'Disable' : 'Enable'}
                </button>
                <button class="btn btn-small" onclick="viewEvents('${webhook.id}')">View Events</button>
                <button class="btn btn-danger btn-small" onclick="deleteWebhook('${webhook.id}')">Delete</button>
                <button class="btn btn-small" onclick="regenerateSecret('${webhook.id}')">New Secret</button>
            </div>
        </div>
    `).join('');
}

function renderRecentEvents(events) {
    const container = document.getElementById('recentEvents');

    if (events.length === 0) {
        container.innerHTML = '<p>No events yet...</p>';
        return;
    }

    container.innerHTML = events.map(event => `
        <div style="padding: 10px; border: 1px solid #e5e7eb; border-radius: 4px; margin-bottom: 8px; font-size: 12px;">
            <div><strong>${event.endpoint}</strong></div>
            <div style="color: #6b7280;">${event.event_type || 'Unknown'} • ${new Date(event.timestamp).toLocaleString()}</div>
        </div>
    `).join('');
}

function updateStats(webhooks) {
    document.getElementById('totalWebhooks').textContent = webhooks.length;
    document.getElementById('activeWebhooks').textContent = webhooks.filter(w => w.is_active).length;

    const totalEvents = webhooks.reduce((sum, w) => sum + (w.event_count || 0), 0);
    document.getElementById('totalEvents').textContent = totalEvents;
}

async function createWebhook() {
    const form = document.getElementById('createWebhookForm');
    const formData = new FormData(form);

    const data = {
        name: formData.get('name'),
        endpoint: formData.get('endpoint'),
        description: formData.get('description') || null
    };

    try {
        await apiCall('/api/webhooks', {
            method: 'POST',
            body: JSON.stringify(data)
        });

        form.reset();
        await loadWebhooks();
        alert('Webhook created successfully!');
    } catch (error) {
        console.error('Failed to create webhook:', error);
    }
}

async function toggleWebhook(id, isActive) {
    try {
        await apiCall(`/api/webhooks/${id}`, {
            method: 'PUT',
            body: JSON.stringify({ is_active: isActive })
        });

        await loadWebhooks();
    } catch (error) {
        console.error('Failed to toggle webhook:', error);
    }
}

async function regenerateSecret(id) {
    if (!confirm('Generate a new secret? The old secret will stop working.')) return;

    try {
        await apiCall(`/api/webhooks/${id}/regenerate-secret`, {
            method: 'POST'
        });

        await loadWebhooks();
        alert('New secret generated!');
    } catch (error) {
        console.error('Failed to regenerate secret:', error);
    }
}

function deleteWebhook(id) {
    deleteWebhookId = id;
    document.getElementById('deleteModal').style.display = 'block';
}

function closeDeleteModal() {
    document.getElementById('deleteModal').style.display = 'none';
    deleteWebhookId = null;
}

async function confirmDelete() {
    if (!deleteWebhookId) return;

    try {
        await apiCall(`/api/webhooks/${deleteWebhookId}`, {
            method: 'DELETE'
        });

        closeDeleteModal();
        await loadWebhooks();
        alert('Webhook deleted successfully!');
    } catch (error) {
        console.error('Failed to delete webhook:', error);
    }
}

function viewEvents(webhookId) {
    window.open(`/events?webhook=${webhookId}`, '_blank');
}

document.getElementById('createWebhookForm').addEventListener('submit', function(e) {
    e.preventDefault();
    createWebhook();
});

async function checkAuth() {
    return true; // Temporarily disable auth check
}

document.addEventListener('DOMContentLoaded', function() {
    console.log('DOM loaded, starting to load data...');
    loadWebhooks();
    loadRecentEvents();

    setInterval(() => {
        loadRecentEvents();
    }, 30000);
});

window.onclick = function(event) {
    const modal = document.getElementById('deleteModal');
    if (event.target === modal) {
        closeDeleteModal();
    }
}