// API Response wrapper
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}

// Upstream Resolver
export interface UpstreamResolver {
  id: number;
  ip_address: string;
  domain: string;
  created_at: number;
  updated_at: number;
}

export interface CreateResolverRequest {
  ip_address: string;
  domain: string;
}

export interface UpdateResolverRequest {
  ip_address?: string;
  domain?: string;
}

// Priority Rule
export interface PriorityRule {
  id: number;
  domain: string;
  client_filter: string | null;
  target_ip: string;
  active: boolean;
  created_at: number;
  updated_at: number;
}

export interface CreateRuleRequest {
  domain: string;
  client_filter?: string | null;
  target_ip: string;
  active?: boolean;
}

export interface UpdateRuleRequest {
  domain?: string;
  client_filter?: string | null;
  target_ip?: string;
  active?: boolean;
}

// Statistics
export interface Statistic {
  id: number;
  rule_id: number;
  client_ip: string;
  requested_domain: string;
  original_ip: string | null;
  overridden_ip: string;
  timestamp: number;
}

export interface StatsSummary {
  total_hits: number;
  unique_clients: number;
  active_rules: number;
}

// API Results
export interface CreateResponse {
  id: number;
}

export interface UpdateResponse {
  updated: boolean;
}

export interface DeleteResponse {
  deleted: boolean;
}
