export const PROPERTY_TYPES = Object.freeze([
  { key: 'string', label: 'String', mark: 'STR', ariaLabel: 'String datatype' },
  { key: 'integer', label: 'Integer', mark: 'INT', ariaLabel: 'Integer datatype' },
  { key: 'byte', label: 'Byte', mark: 'BYTE', ariaLabel: 'Byte datatype' },
  { key: 'boolean', label: 'Boolean', mark: 'BOOL', ariaLabel: 'Boolean datatype' },
  { key: 'decimal', label: 'Decimal', mark: 'DEC', ariaLabel: 'Decimal datatype' },
  { key: 'float', label: 'Float', mark: 'FLT', ariaLabel: 'Floating-point datatype' },
  { key: 'long', label: 'Long', mark: 'LONG', ariaLabel: 'Long integer datatype' },
  { key: 'date', label: 'Date', mark: 'DATE', ariaLabel: 'Date datatype' },
  { key: 'time', label: 'Time', mark: 'TIME', ariaLabel: 'Time datatype' },
  { key: 'datetime', label: 'Date and time', mark: 'DT', ariaLabel: 'Date and time datatype' },
  { key: 'uuid', label: 'UUID', mark: 'UUID', ariaLabel: 'UUID datatype' },
  { key: 'uri', label: 'URI', mark: 'URI', ariaLabel: 'URI datatype' },
  { key: 'json', label: 'JSON', mark: 'JSON', ariaLabel: 'JSON datatype' },
  { key: 'binary', label: 'Binary', mark: 'BIN', ariaLabel: 'Binary datatype' }
]);

export const MUTATION_TYPES = Object.freeze([
  { key: 'query', label: 'Query', mark: '←ƒ', ariaLabel: 'Query function that returns items from the collection' },
  { key: 'function', label: 'Function', mark: 'ƒ→', ariaLabel: 'Function that updates the collection' },
  { key: 'transformation', label: 'Transformation', mark: 'ƒ', ariaLabel: 'Function that modifies internal domain object state' }
]);

export const COLLECTION_TYPES = Object.freeze([
  { key: 'scalar', label: 'Scalar (0..1)', mark: '', ariaLabel: 'Scalar intermediate type, zero or one target', maxTargets: 1 },
  { key: 'stack', label: 'Stack', mark: '☰', ariaLabel: 'Stack intermediate type' },
  { key: 'queue', label: 'Queue', mark: '|||', ariaLabel: 'Queue intermediate type' },
  { key: 'set', label: 'Set', mark: '{}', ariaLabel: 'Set intermediate type' },
  { key: 'map', label: 'Map', mark: '{:}', ariaLabel: 'Map intermediate type' }
]);

export const OBJECT_LINK_TYPES = Object.freeze([
  { key: 'sameAs', label: 'Same as', mark: '=', ariaLabel: 'Same as object link' },
  { key: 'subclassOf', label: 'Subclass of', mark: '⊂', ariaLabel: 'Subclass of object link' }
]);

export const TYPE_REGISTRY = Object.freeze({
  property: { field: 'dataType', default: 'string', entries: PROPERTY_TYPES },
  mutationNode: { field: 'mutationType', default: 'query', entries: MUTATION_TYPES },
  edgeNode: { field: 'collectionType', default: 'scalar', entries: COLLECTION_TYPES },
  objectLink: { field: 'linkType', default: 'sameAs', entries: OBJECT_LINK_TYPES }
});

export const typeDefinition = (kind, key) => TYPE_REGISTRY[kind]?.entries.find(entry => entry.key === key);
export const defaultSubtype = kind => TYPE_REGISTRY[kind]?.default;
export const subtypeField = kind => TYPE_REGISTRY[kind]?.field;
export const withDefaultSubtype = (kind, value = {}) => {
  const field = subtypeField(kind);
  return field ? { ...value, [field]: value[field] ?? defaultSubtype(kind) } : { ...value };
};
