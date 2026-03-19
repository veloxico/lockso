-- Fix FK constraints to allow user deletion
-- Change creator_id references from RESTRICT (default) to SET NULL

-- vaults.creator_id → SET NULL on user delete
ALTER TABLE vaults ALTER COLUMN creator_id DROP NOT NULL;
ALTER TABLE vaults DROP CONSTRAINT IF EXISTS vaults_creator_id_fkey;
ALTER TABLE vaults ADD CONSTRAINT vaults_creator_id_fkey
    FOREIGN KEY (creator_id) REFERENCES users(id) ON DELETE SET NULL;

-- items.creator_id → SET NULL on user delete
ALTER TABLE items ALTER COLUMN creator_id DROP NOT NULL;
ALTER TABLE items DROP CONSTRAINT IF EXISTS items_creator_id_fkey;
ALTER TABLE items ADD CONSTRAINT items_creator_id_fkey
    FOREIGN KEY (creator_id) REFERENCES users(id) ON DELETE SET NULL;
