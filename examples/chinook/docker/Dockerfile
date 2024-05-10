FROM postgres:16.2-alpine3.19

ENV PGTAP_VERSION v1.3.3

RUN apk -U add \
    alpine-sdk \
    perl \
    && git clone https://github.com/theory/pgtap \
    && cd pgtap \
    && git checkout ${PGTAP_VERSION} \
    && make \
    && make install

RUN apk -U add \
    build-base \
    perl-dev \
    && cpan TAP::Parser::SourceHandler::pgTAP \
    && apk del -r build-base

ADD https://github.com/lerocha/chinook-database/releases/download/v1.4.5/Chinook_PostgreSql_SerialPKs.sql /docker-entrypoint-initdb.d/

RUN echo "create extension pgtap;" >> /docker-entrypoint-initdb.d/Chinook_PostgreSql_SerialPKs.sql

RUN chmod a+r /docker-entrypoint-initdb.d/*.sql

RUN mkdir /tmp/tapestry-chinook

RUN mkdir /tmp/tapestry-chinook/tests

COPY run-tests /tmp/tapestry-chinook/

RUN chmod a+x /tmp/tapestry-chinook/run-tests

EXPOSE 5432
