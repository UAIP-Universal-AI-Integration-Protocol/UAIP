export interface HealthResponse {
    status: string;
    version: string;
    timestamp: string;
}

export interface DeviceInfo {
    device_id: string;
    name: string;
    device_type: string; // 'sensor' | 'camera' | 'robot' | 'actuator' etc.
    status: 'online' | 'offline' | 'maintenance' | 'error';
    last_seen: string | null;
}

export interface DeviceListResponse {
    devices: DeviceInfo[];
    total: number;
}

export interface DeviceRegistrationRequest {
    device_id: string;
    device_type: string;
    name: string;
    manufacturer?: string;
    model?: string;
    capabilities: string[];
}

export interface CommandRequest {
    action: string;
    parameters?: Record<string, any>;
    priority?: 'low' | 'normal' | 'high' | 'critical';
}

export interface CommandResponse {
    message_id: string;
    status: string;
    queued_at: string;
}

export interface AiAgent {
    id: string;
    name: string;
    status: 'active' | 'inactive' | 'learning';
    capabilities: string[];
    last_active: string;
}

export interface MediaItem {
    id: string;
    filename: string;
    type: 'image' | 'video' | 'audio';
    size: number;
    uploaded_at: string;
    url: string;
}
