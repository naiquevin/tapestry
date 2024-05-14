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

COPY run-tests /usr/local/bin/

RUN chmod a+x /usr/local/bin/run-tests

RUN mkdir /tmp/tapestry-data

EXPOSE 5432
