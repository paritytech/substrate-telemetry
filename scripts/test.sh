yarn && tsc -p packages/common && node packages/common/test | tap-spec && cd packages/frontend && yarn test && cd ../../
