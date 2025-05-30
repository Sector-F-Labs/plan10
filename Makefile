
.PHONY: all clean push

push:
	echo "time to push data"
	scp neo4j.conf divan@plan10:/usr/local/Cellar/neo4j/2025.04.0/libexec/conf/neo4j.conf

