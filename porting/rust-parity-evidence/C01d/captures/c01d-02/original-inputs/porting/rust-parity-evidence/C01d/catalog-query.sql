-- Coordinator-reviewed candidate; psql -X -q -A -t -v ON_ERROR_STOP=1.
-- Bind artifact_sha256, target_revision, source_label_revision, query_sha256.
BEGIN TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY;
SET LOCAL statement_timeout = '20s';
SET LOCAL search_path = pg_catalog, public;
WITH RECURSIVE selected_relations(oid) AS (
 SELECT c.oid FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
 WHERE c.relkind IN ('r','p') AND n.nspname !~ '^pg_' AND n.nspname<>'information_schema'
 UNION
 SELECT k.confrelid FROM pg_constraint k JOIN selected_relations r ON r.oid=k.conrelid WHERE k.confrelid<>0
), relations AS (
 SELECT c.*, n.nspname FROM selected_relations r JOIN pg_class c ON c.oid=r.oid JOIN pg_namespace n ON n.oid=c.relnamespace
), history AS (
 SELECT (row_number() OVER (ORDER BY orderexecuted,dateexecuted,id,author,filename)-1)::bigint AS position,
 id,author,filename,dateexecuted::text,orderexecuted,exectype,md5sum,description,comments,tag,liquibase,contexts,labels,deployment_id
 FROM public.databasechangelog
)
SELECT jsonb_build_object(
 'schema_version',1,
 'capture',jsonb_build_object(
  'artifact_sha256',:'artifact_sha256','target_revision',:'target_revision',
  'source_label_revision',:'source_label_revision','query_sha256',:'query_sha256',
  'database',current_database(),'server_version',current_setting('server_version'),
  'server_version_num',current_setting('server_version_num')::integer,
  'schemas',(SELECT coalesce(jsonb_agg(s.nspname ORDER BY s.nspname),'[]') FROM (SELECT DISTINCT nspname FROM relations) s),
  'selected_relation_kinds',jsonb_build_array('r','p'),'search_path',current_setting('search_path'),
  'transaction_isolation',current_setting('transaction_isolation'),'transaction_read_only',current_setting('transaction_read_only'),
  'changelog_table',jsonb_build_object('schema','public','name','databasechangelog'),'captured_at',transaction_timestamp()::text),
 'tables',(SELECT coalesce(jsonb_agg(jsonb_build_object(
  'oid',r.oid::bigint,'schema',r.nspname,'name',r.relname,'relkind',r.relkind::text,'relpersistence',r.relpersistence::text)
  ORDER BY r.nspname,r.relname),'[]') FROM relations r),
 'columns',(SELECT coalesce(jsonb_agg(jsonb_build_object(
  'table_oid',a.attrelid::bigint,'attnum',a.attnum,'name',a.attname,'type_oid',a.atttypid::bigint,
  'formatted_type',format_type(a.atttypid,a.atttypmod),'not_null',a.attnotnull,
  'default_expr',pg_get_expr(d.adbin,d.adrelid),'identity',a.attidentity::text,'generated',a.attgenerated::text,
  'collation',CASE WHEN a.attcollation=0 THEN NULL ELSE quote_ident(cn.nspname)||'.'||quote_ident(co.collname) END)
  ORDER BY a.attrelid,a.attnum),'[]') FROM pg_attribute a JOIN relations r ON r.oid=a.attrelid
  LEFT JOIN pg_attrdef d ON d.adrelid=a.attrelid AND d.adnum=a.attnum
  LEFT JOIN pg_collation co ON co.oid=a.attcollation LEFT JOIN pg_namespace cn ON cn.oid=co.collnamespace
  WHERE a.attnum>0 AND NOT a.attisdropped),
 'constraints',(SELECT coalesce(jsonb_agg(jsonb_build_object(
  'oid',k.oid::bigint,'table_oid',k.conrelid::bigint,'name',k.conname,'contype',k.contype::text,
  'definition',pg_get_constraintdef(k.oid,true),'validated',k.convalidated,'deferrable',k.condeferrable,
  'initially_deferred',k.condeferred,'referenced_table_oid',nullif(k.confrelid,0)::bigint,'index_oid',nullif(k.conindid,0)::bigint,
  'columns',coalesce(to_jsonb(k.conkey),'[]'),'referenced_columns',coalesce(to_jsonb(k.confkey),'[]'))
  ORDER BY k.conrelid,k.conname),'[]') FROM pg_constraint k JOIN relations r ON r.oid=k.conrelid),
 'indexes',(SELECT coalesce(jsonb_agg(jsonb_build_object(
  'oid',i.indexrelid::bigint,'table_oid',i.indrelid::bigint,'schema',n.nspname,'name',c.relname,
  'access_method',am.amname,'definition',pg_get_indexdef(i.indexrelid),'unique',i.indisunique,'primary',i.indisprimary,
  'valid',i.indisvalid,'ready',i.indisready,'predicate',pg_get_expr(i.indpred,i.indrelid),
  'expressions',pg_get_expr(i.indexprs,i.indrelid),'key_attnums',to_jsonb(i.indkey::smallint[]),'key_count',i.indnkeyatts)
  ORDER BY n.nspname,c.relname),'[]') FROM pg_index i JOIN relations r ON r.oid=i.indrelid
  JOIN pg_class c ON c.oid=i.indexrelid JOIN pg_namespace n ON n.oid=c.relnamespace JOIN pg_am am ON am.oid=c.relam),
 'changelog',(SELECT coalesce(jsonb_agg(to_jsonb(h) ORDER BY h.position),'[]') FROM history h)
);
ROLLBACK;
