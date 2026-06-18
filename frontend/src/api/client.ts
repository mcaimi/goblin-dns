import axios from 'axios';
import type {
  ApiResponse,
  UpstreamResolver,
  CreateResolverRequest,
  UpdateResolverRequest,
  PriorityRule,
  CreateRuleRequest,
  UpdateRuleRequest,
  Statistic,
  StatsSummary,
  CreateResponse,
  UpdateResponse,
  DeleteResponse,
} from '../types';

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080/api/v1';

const api = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

// ========== Upstream Resolvers ==========

export const resolversApi = {
  getAll: () =>
    api.get<ApiResponse<UpstreamResolver[]>>('/resolvers').then((res) => res.data),

  getById: (id: number) =>
    api.get<ApiResponse<UpstreamResolver>>(`/resolvers/${id}`).then((res) => res.data),

  create: (data: CreateResolverRequest) =>
    api.post<ApiResponse<CreateResponse>>('/resolvers', data).then((res) => res.data),

  update: (id: number, data: UpdateResolverRequest) =>
    api.put<ApiResponse<UpdateResponse>>(`/resolvers/${id}`, data).then((res) => res.data),

  delete: (id: number) =>
    api.delete<ApiResponse<DeleteResponse>>(`/resolvers/${id}`).then((res) => res.data),
};

// ========== Priority Rules ==========

export const rulesApi = {
  getAll: () =>
    api.get<ApiResponse<PriorityRule[]>>('/rules').then((res) => res.data),

  getById: (id: number) =>
    api.get<ApiResponse<PriorityRule>>(`/rules/${id}`).then((res) => res.data),

  create: (data: CreateRuleRequest) =>
    api.post<ApiResponse<CreateResponse>>('/rules', data).then((res) => res.data),

  update: (id: number, data: UpdateRuleRequest) =>
    api.put<ApiResponse<UpdateResponse>>(`/rules/${id}`, data).then((res) => res.data),

  toggle: (id: number) =>
    api.patch<ApiResponse<UpdateResponse>>(`/rules/${id}/toggle`).then((res) => res.data),

  delete: (id: number) =>
    api.delete<ApiResponse<DeleteResponse>>(`/rules/${id}`).then((res) => res.data),
};

// ========== Statistics ==========

export const statsApi = {
  getAll: (limit = 100, offset = 0) =>
    api.get<ApiResponse<Statistic[]>>('/stats', { params: { limit, offset } }).then((res) => res.data),

  getByRule: (ruleId: number) =>
    api.get<ApiResponse<Statistic[]>>(`/stats/rules/${ruleId}`).then((res) => res.data),

  getSummary: () =>
    api.get<ApiResponse<StatsSummary>>('/stats/summary').then((res) => res.data),
};

// ========== Health Check ==========

export const healthApi = {
  check: () => api.get('/health'),
};
