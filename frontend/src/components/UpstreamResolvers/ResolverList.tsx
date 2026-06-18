import { useState } from 'react';
import { useResolvers, useCreateResolver, useDeleteResolver } from '../../hooks/useResolvers';
import type { CreateResolverRequest } from '../../types';

export default function ResolverList() {
  const { data, isLoading, error } = useResolvers();
  const createMutation = useCreateResolver();
  const deleteMutation = useDeleteResolver();

  const [showForm, setShowForm] = useState(false);
  const [formData, setFormData] = useState<CreateResolverRequest>({
    ip_address: '',
    domain: '',
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    await createMutation.mutateAsync(formData);
    setFormData({ ip_address: '', domain: '' });
    setShowForm(false);
  };

  const handleDelete = async (id: number) => {
    if (confirm('Are you sure you want to delete this resolver?')) {
      await deleteMutation.mutateAsync(id);
    }
  };

  if (isLoading) return <div>Loading resolvers...</div>;
  if (error) return <div>Error loading resolvers: {error.message}</div>;

  const resolvers = data?.data || [];

  return (
    <div>
      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '20px' }}>
        <h2>Upstream Resolvers</h2>
        <button onClick={() => setShowForm(!showForm)}>
          {showForm ? 'Cancel' : 'Add Resolver'}
        </button>
      </div>

      {showForm && (
        <form onSubmit={handleSubmit} style={{ marginBottom: '20px', padding: '15px', border: '1px solid #ccc' }}>
          <div style={{ marginBottom: '10px' }}>
            <label>
              IP Address:
              <input
                type="text"
                value={formData.ip_address}
                onChange={(e) => setFormData({ ...formData, ip_address: e.target.value })}
                placeholder="8.8.8.8:53"
                required
                style={{ marginLeft: '10px', padding: '5px' }}
              />
            </label>
          </div>
          <div style={{ marginBottom: '10px' }}>
            <label>
              Domain:
              <input
                type="text"
                value={formData.domain}
                onChange={(e) => setFormData({ ...formData, domain: e.target.value })}
                placeholder="core or example.com"
                required
                style={{ marginLeft: '10px', padding: '5px' }}
              />
            </label>
          </div>
          <button type="submit" disabled={createMutation.isPending}>
            {createMutation.isPending ? 'Creating...' : 'Create Resolver'}
          </button>
        </form>
      )}

      <table style={{ width: '100%', borderCollapse: 'collapse' }}>
        <thead>
          <tr style={{ backgroundColor: '#f0f0f0' }}>
            <th style={{ padding: '10px', textAlign: 'left', border: '1px solid #ccc' }}>ID</th>
            <th style={{ padding: '10px', textAlign: 'left', border: '1px solid #ccc' }}>IP Address</th>
            <th style={{ padding: '10px', textAlign: 'left', border: '1px solid #ccc' }}>Domain</th>
            <th style={{ padding: '10px', textAlign: 'left', border: '1px solid #ccc' }}>Actions</th>
          </tr>
        </thead>
        <tbody>
          {resolvers.map((resolver) => (
            <tr key={resolver.id}>
              <td style={{ padding: '10px', border: '1px solid #ccc' }}>{resolver.id}</td>
              <td style={{ padding: '10px', border: '1px solid #ccc' }}>{resolver.ip_address}</td>
              <td style={{ padding: '10px', border: '1px solid #ccc' }}>{resolver.domain}</td>
              <td style={{ padding: '10px', border: '1px solid #ccc' }}>
                <button onClick={() => handleDelete(resolver.id)} style={{ color: 'red' }}>
                  Delete
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>

      {resolvers.length === 0 && (
        <div style={{ padding: '20px', textAlign: 'center', color: '#666' }}>
          No upstream resolvers configured
        </div>
      )}
    </div>
  );
}
