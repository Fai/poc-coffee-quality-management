const API_BASE = import.meta.env.VITE_API_URL || '/api';
const AI_API_BASE = import.meta.env.VITE_AI_API_URL || '/ai';

class ApiClient {
  private token: string | null = null;
  private baseUrl: string;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }

  setToken(token: string | null) {
    this.token = token;
  }

  private async request<T>(path: string, options: RequestInit = {}): Promise<T> {
    const headers: HeadersInit = { 'Content-Type': 'application/json', ...options.headers };
    if (this.token) headers['Authorization'] = `Bearer ${this.token}`;

    const res = await fetch(`${this.baseUrl}${path}`, { ...options, headers });
    if (!res.ok) {
      const error = await res.json().catch(() => ({ message: 'Request failed' }));
      throw new Error(error.error?.message_en || error.message || error.error || 'Request failed');
    }
    return res.json();
  }

  get<T>(path: string) { return this.request<T>(path); }
  post<T>(path: string, data?: unknown) { return this.request<T>(path, { method: 'POST', body: JSON.stringify(data) }); }
  put<T>(path: string, data?: unknown) { return this.request<T>(path, { method: 'PUT', body: JSON.stringify(data) }); }
  delete<T>(path: string) { return this.request<T>(path, { method: 'DELETE' }); }
}

export const api = new ApiClient(API_BASE);
const aiApi = new ApiClient(AI_API_BASE);

// API Types
export interface DashboardMetrics {
  total_lots: number;
  active_lots: number;
  total_inventory_kg: number;
  avg_cupping_score: number | null;
  pending_alerts: number;
  recent_harvests: number;
  expiring_certifications: number;
}

export interface Plot {
  id: string;
  name: string;
  area_rai: number;
  altitude_meters: number | null;
  shade_coverage_percent: number | null;
  varieties: { variety: string; planting_date: string | null }[];
}

export interface Lot {
  id: string;
  traceability_code: string;
  name: string;
  stage: string;
  current_weight_kg: number;
  created_at: string;
}

export interface InventoryTransaction {
  id: string;
  lot_id: string;
  transaction_type: string;
  quantity_kg: number;
  created_at: string;
}

// API Functions
export const dashboardApi = {
  getMetrics: () => api.get<DashboardMetrics>('/reports/dashboard'),
};

export const plotsApi = {
  list: () => api.get<Plot[]>('/plots'),
  get: (id: string) => api.get<Plot>(`/plots/${id}`),
  create: (data: Partial<Plot>) => api.post<Plot>('/plots', data),
  update: (id: string, data: Partial<Plot>) => api.put<Plot>(`/plots/${id}`, data),
  delete: (id: string) => api.delete(`/plots/${id}`),
};

export const lotsApi = {
  list: () => api.get<Lot[]>('/lots'),
  get: (id: string) => api.get<Lot>(`/lots/${id}`),
  create: (data: Partial<Lot>) => api.post<Lot>('/lots', data),
};

export const inventoryApi = {
  getTransactions: (lotId: string) => api.get<InventoryTransaction[]>(`/inventory/lots/${lotId}/transactions`),
  getSummary: () => api.get('/inventory/summary'),
};

// AI Defect Detection - supports both binary classification and YOLO object detection
export interface DefectDetectionResult {
  request_id: string;
  detection: {
    // Binary classification (old model)
    is_defective?: boolean;
    defect_probability?: number;
    confidence_score?: number;
    // YOLO object detection (new model)
    total_defects?: number;
    defect_counts?: Record<string, number>;
    detections?: Array<{ class: string; confidence: number; bbox: number[] }>;
    // Common
    processing_time_ms: number;
    model: string;
    note?: string;
  };
  suggested_grade: string;
}

export const defectApi = {
  detect: (imageBase64: string) => aiApi.post<DefectDetectionResult>('/detect', { image_base64: imageBase64 }),
  detectYolo: (imageBase64: string) => aiApi.post<DefectDetectionResult>('/yolo', { image_base64: imageBase64 }),
};
