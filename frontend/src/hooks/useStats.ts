import { useQuery } from '@tanstack/react-query';
import { statsApi } from '../api/client';

export const useStats = (limit = 100, offset = 0, refetchInterval: number | false = false) => {
  return useQuery({
    queryKey: ['stats', limit, offset],
    queryFn: () => statsApi.getAll(limit, offset),
    refetchInterval,
  });
};

export const useStatsByRule = (ruleId: number) => {
  return useQuery({
    queryKey: ['stats', 'rule', ruleId],
    queryFn: () => statsApi.getByRule(ruleId),
    enabled: !!ruleId,
  });
};

export const useStatsSummary = () => {
  return useQuery({
    queryKey: ['stats', 'summary'],
    queryFn: statsApi.getSummary,
    refetchInterval: 10000, // Refetch every 10 seconds
  });
};
