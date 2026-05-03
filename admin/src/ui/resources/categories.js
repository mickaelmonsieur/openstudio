export const categoriesResource = {
  name: 'categories',
  title: 'Categories',
  group: 'Media',
  endpoint: '/api/categories',
  primaryKey: 'id',
  columns: [
    { key: 'id', label: 'ID', width: '80px' },
    { key: 'name', label: 'Name' }
  ],
  form: [
    {
      key: 'name',
      label: 'Name',
      type: 'text',
      required: true,
      maxLength: 32
    }
  ],
  actions: {
    add: true,
    edit: true,
    delete: true
  }
};

export const subcategoriesResource = {
  name: 'subcategories',
  title: 'Subcategories',
  endpoint: '/api/subcategories',
  primaryKey: 'id',
  columns: [
    { key: 'id', label: 'ID', width: '80px' },
    { key: 'name', label: 'Name' },
    { key: 'hidden', label: 'Hidden', width: '110px' }
  ],
  form: [
    {
      key: 'name',
      label: 'Name',
      type: 'text',
      required: true,
      maxLength: 32
    },
    {
      key: 'hidden',
      label: 'Hidden',
      type: 'checkbox'
    }
  ],
  actions: {
    add: true,
    edit: true,
    delete: true
  }
};
