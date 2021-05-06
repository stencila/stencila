module.exports = {
  sidebar: [
    'index',
    {
      type: 'category',
      label: 'Schema',
      items: [
        'schema/index',
        ...require('./schema/docs/categories'),
        'schema/docs/index'
      ]
    },
  ],
}
