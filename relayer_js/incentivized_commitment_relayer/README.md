ingest:
  start
  query ethereum for latest nonce
  query parachain for latest nonce
  catch up difference and continue watching
  ingest commitments into queue

processing:
 take commitment from queue
 submit to ethereum
 ensure acceptance
