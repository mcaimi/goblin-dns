import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { rulesApi } from '../api/client';
import type { CreateRuleRequest, UpdateRuleRequest } from '../types';

export const useRules = () => {
  return useQuery({
    queryKey: ['rules'],
    queryFn: rulesApi.getAll,
  });
};

export const useRule = (id: number) => {
  return useQuery({
    queryKey: ['rule', id],
    queryFn: () => rulesApi.getById(id),
    enabled: !!id,
  });
};

export const useCreateRule = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: CreateRuleRequest) => rulesApi.create(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['rules'] });
      queryClient.invalidateQueries({ queryKey: ['stats'] });
    },
  });
};

export const useUpdateRule = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, data }: { id: number; data: UpdateRuleRequest }) =>
      rulesApi.update(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['rules'] });
    },
  });
};

export const useToggleRule = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: number) => rulesApi.toggle(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['rules'] });
      queryClient.invalidateQueries({ queryKey: ['stats'] });
    },
  });
};

export const useDeleteRule = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: number) => rulesApi.delete(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['rules'] });
      queryClient.invalidateQueries({ queryKey: ['stats'] });
    },
  });
};
