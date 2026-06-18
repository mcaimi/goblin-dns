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

  if (isLoading) return <div className="loading">Loading resolvers...</div>;
  if (error) return <div className="error">Error loading resolvers: {error.message}</div>;

  const resolvers = data?.data || [];

  return (
    <div>
      {showForm && (
        <div className="card">
          <form onSubmit={handleSubmit}>
            <div className="form-group">
              <label>IP Address</label>
              <input
                type="text"
                value={formData.ip_address}
                onChange={(e) => setFormData({ ...formData, ip_address: e.target.value })}
                placeholder="8.8.8.8:53"
                required
              />
            </div>
            <div className="form-group">
              <label>Domain</label>
              <input
                type="text"
                value={formData.domain}
                onChange={(e) => setFormData({ ...formData, domain: e.target.value })}
                placeholder="core or example.com"
                required
              />
            </div>
            <button type="submit" className="btn-primary" disabled={createMutation.isPending}>
              {createMutation.isPending ? 'Creating...' : 'Create Resolver'}
            </button>
            <button type="button" className="btn-secondary" onClick={() => setShowForm(false)} style={{ marginLeft: '10px' }}>
              Cancel
            </button>
          </form>
        </div>
      )}

      <div className="card">
        <div className="card-header">
          <h2 className="card-title">Upstream Resolvers</h2>
          <button className="btn-primary" onClick={() => setShowForm(!showForm)}>
            {showForm ? 'Cancel' : 'Add Resolver'}
          </button>
        </div>

        <table>
          <thead>
            <tr>
              <th>ID</th>
              <th>IP Address</th>
              <th>Domain</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {resolvers.map((resolver) => (
              <tr key={resolver.id}>
                <td>{resolver.id}</td>
                <td>{resolver.ip_address}</td>
                <td>{resolver.domain}</td>
                <td>
                  <button className="btn-danger" onClick={() => handleDelete(resolver.id)}>
                    Delete
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>

        {resolvers.length === 0 && (
          <p style={{ textAlign: 'center', color: '#95a5a6', padding: '20px' }}>
            No upstream resolvers configured
          </p>
        )}
      </div>
    </div>
  );
}
