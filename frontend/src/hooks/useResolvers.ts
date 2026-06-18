import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { resolversApi } from '../api/client';
import type { CreateResolverRequest, UpdateResolverRequest } from '../types';

export const useResolvers = () => {
  return useQuery({
    queryKey: ['resolvers'],
    queryFn: resolversApi.getAll,
  });
};

export const useResolver = (id: number) => {
  return useQuery({
    queryKey: ['resolver', id],
    queryFn: () => resolversApi.getById(id),
    enabled: !!id,
  });
};

export const useCreateResolver = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: CreateResolverRequest) => resolversApi.create(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['resolvers'] });
    },
  });
};

export const useUpdateResolver = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, data }: { id: number; data: UpdateResolverRequest }) =>
      resolversApi.update(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['resolvers'] });
    },
  });
};

export const useDeleteResolver = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: number) => resolversApi.delete(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['resolvers'] });
    },
  });
};
