import { useState } from 'react';
import { useRules, useCreateRule, useDeleteRule, useToggleRule } from '../../hooks/useRules';
import type { CreateRuleRequest } from '../../types';

export default function RuleList() {
  const { data, isLoading, error } = useRules();
  const createMutation = useCreateRule();
  const deleteMutation = useDeleteRule();
  const toggleMutation = useToggleRule();

  const [showForm, setShowForm] = useState(false);
  const [formData, setFormData] = useState<CreateRuleRequest>({
    domain: '',
    client_filter: '',
    target_ip: '',
    active: true,
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const data = {
      ...formData,
      client_filter: formData.client_filter || null,
    };
    await createMutation.mutateAsync(data);
    setFormData({ domain: '', client_filter: '', target_ip: '', active: true });
    setShowForm(false);
  };

  const handleDelete = async (id: number) => {
    if (confirm('Are you sure you want to delete this rule?')) {
      await deleteMutation.mutateAsync(id);
    }
  };

  const handleToggle = async (id: number) => {
    await toggleMutation.mutateAsync(id);
  };

  if (isLoading) return <div className="loading">Loading rules...</div>;
  if (error) return <div className="error">Error loading rules: {error.message}</div>;

  const rules = data?.data || [];

  return (
    <div>
      {showForm && (
        <div className="card">
          <form onSubmit={handleSubmit}>
            <div className="form-group">
              <label>Domain</label>
              <input
                type="text"
                value={formData.domain}
                onChange={(e) => setFormData({ ...formData, domain: e.target.value })}
                placeholder="evil.com"
                required
              />
            </div>
            <div className="form-group">
              <label>Client Filter (optional)</label>
              <input
                type="text"
                value={formData.client_filter || ''}
                onChange={(e) => setFormData({ ...formData, client_filter: e.target.value })}
                placeholder="192.168.1.0/24 or empty for all"
              />
            </div>
            <div className="form-group">
              <label>Target IP</label>
              <input
                type="text"
                value={formData.target_ip}
                onChange={(e) => setFormData({ ...formData, target_ip: e.target.value })}
                placeholder="127.0.0.1"
                required
              />
            </div>
            <div className="checkbox-container">
              <input
                type="checkbox"
                checked={formData.active}
                onChange={(e) => setFormData({ ...formData, active: e.target.checked })}
              />
              <label>Active</label>
            </div>
            <button type="submit" className="btn-primary" disabled={createMutation.isPending}>
              {createMutation.isPending ? 'Creating...' : 'Create Rule'}
            </button>
            <button type="button" className="btn-secondary" onClick={() => setShowForm(false)} style={{ marginLeft: '10px' }}>
              Cancel
            </button>
          </form>
        </div>
      )}

      <div className="card">
        <div className="card-header">
          <h2 className="card-title">Priority Rules</h2>
          <button className="btn-primary" onClick={() => setShowForm(!showForm)}>
            {showForm ? 'Cancel' : 'Add Rule'}
          </button>
        </div>

        <table>
          <thead>
            <tr>
              <th>ID</th>
              <th>Domain</th>
              <th>Client Filter</th>
              <th>Target IP</th>
              <th>Status</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {rules.map((rule) => (
              <tr key={rule.id}>
                <td>{rule.id}</td>
                <td>{rule.domain}</td>
                <td>{rule.client_filter || 'All clients'}</td>
                <td>{rule.target_ip}</td>
                <td>
                  <span className={rule.active ? 'badge badge-success' : 'badge badge-secondary'}>
                    {rule.active ? 'Active' : 'Inactive'}
                  </span>
                </td>
                <td>
                  <button className="btn-secondary" onClick={() => handleToggle(rule.id)}>
                    {rule.active ? 'Deactivate' : 'Activate'}
                  </button>
                  <button className="btn-danger" onClick={() => handleDelete(rule.id)} style={{ marginLeft: '5px' }}>
                    Delete
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>

        {rules.length === 0 && (
          <p style={{ textAlign: 'center', color: '#95a5a6', padding: '20px' }}>
            No priority rules configured
          </p>
        )}
      </div>
    </div>
  );
}
