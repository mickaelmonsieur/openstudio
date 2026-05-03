export const templatesResource = {
  name: 'templates',
  title: 'Templates',
  group: 'Automation',
  endpoint: '/api/templates',
  primaryKey: 'id',
  columns: [
    { key: 'id',   label: 'ID',   width: '90px' },
    { key: 'name', label: 'Name' }
  ],
  form: [
    { key: 'name', label: 'Name', type: 'text', required: true, maxLength: 32 }
  ],
  actions: { add: true, edit: true, delete: true }
};
