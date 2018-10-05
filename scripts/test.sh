yarn && tsc -p packages/common && node packages/common/test | tap-spec && cd packages/backend && yarn test && cd ../frontend && yarn test && cd ../../
