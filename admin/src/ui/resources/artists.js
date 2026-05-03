export const artistsResource = {
  name: 'artists',
  title: 'Artists',
  group: 'Media',
  endpoint: '/api/artists',
  searchable: true,
  primaryKey: 'id',
  columns: [
    { key: 'name', label: 'Name' },
    { key: 'last_broadcast_at', label: 'Last Broadcast', width: '190px' }
  ],
  form: [
    {
      key: 'name',
      label: 'Name',
      type: 'text',
      required: true,
      maxLength: 64
    }
  ],
  actions: {
    add: true,
    edit: true,
    delete: true
  }
};
