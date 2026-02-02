class CreateListsAndAddListIdToTasks < ActiveRecord::Migration[8.1]
  def up
    create_table :lists, id: false do |t|
      t.string :id, primary_key: true
      t.string :uid, null: false
      t.string :name, null: false
      t.float :position, null: false
      t.datetime :deleted_at
      t.integer :revision, null: false, default: 0
      t.datetime :created_at, null: false
      t.datetime :updated_at, null: false
    end

    add_index :lists, [:uid, :updated_at]
    add_index :lists, [:uid, :deleted_at]

    add_column :tasks, :list_id, :string
    add_index :tasks, [:uid, :list_id, :status]

    now = Time.zone.now
    uids = execute("SELECT DISTINCT uid FROM tasks").to_a.map { |row| row['uid'] || row[0] }

    uids.each do |uid|
      list_id = "inbox-#{uid}"

      execute <<~SQL
        INSERT INTO lists (id, uid, name, position, revision, created_at, updated_at)
        VALUES (#{connection.quote(list_id)}, #{connection.quote(uid)}, 'Inbox', 1000, 0, #{connection.quote(now)}, #{connection.quote(now)})
      SQL

      execute <<~SQL
        UPDATE tasks
        SET list_id = #{connection.quote(list_id)}, updated_at = #{connection.quote(now)}, revision = revision + 1
        WHERE uid = #{connection.quote(uid)} AND list_id IS NULL
      SQL
    end
  end

  def down
    remove_index :tasks, column: [:uid, :list_id, :status]
    remove_column :tasks, :list_id

    drop_table :lists
  end
end
